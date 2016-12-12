use data::{self, Measurement};
use errors::*;
use gnuplot::{AutoOption, AxesCommon, Figure, PlotOption, Tick};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::cmp;

pub struct Config<'c> {
    pub include_variance: bool,
    pub compute_medians: bool,
    pub compute_normalize: bool,
    pub output_file: &'c str,
    pub filters: &'c [String],
}

pub fn plot(data_file: &str, mut config: Config) -> Result<()> {
    let mut measurements = data::load_measurements(data_file)?;

    // First apply the filters.
    let filters: Vec<_> = try!(config.filters
        .iter()
        .map(|f| {
            let (inverted, text) = if f.starts_with("!") {
                (true, &f[1..])
            } else {
                (false, &f[..])
            };
            Regex::new(text)
                .chain_err(|| format!("filter `{}` not a valid regular expression", f))
                .map(|r| (inverted, r))
        })
        .collect());
    measurements.retain(|m| passes_filters(&filters, m));

    // Convert to medians
    if config.compute_medians {
        measurements = compute_medians(&measurements, config.compute_normalize);
        config.include_variance = true;
    }

    // If there are multiple commits, then we want to use each commit as a point
    // on the X axis.
    if measurements[1..].iter().any(|m| m.commit != measurements[0].commit) {
        return plot_commits_as_x(&measurements, config);
    }

    // If there are multiple test names, use those commits as points
    // on the X axis.
    if measurements[1..].iter().any(|m| m.test != measurements[0].test) {
        return plot_tests_as_x(&measurements, config);
    }

    // Else, use individual measurements as points.
    return plot_indices_as_x(&measurements, config);
}

fn passes_filters(filters: &[(bool, Regex)], m: &Measurement) -> bool {
    if filters.is_empty() {
        true
    } else {
        filters.iter()
            .any(|&(inverted, ref r)| !inverted == (r.is_match(&m.commit) || r.is_match(&m.test)))
    }
}

fn plot_commits_as_x(measurements: &[Measurement], config: Config) -> Result<()> {
    let ref x_axis = compute_x_axis_from_commits(measurements);
    plot_with_x_axis(measurements, &x_axis, config)
}

fn plot_tests_as_x(measurements: &[Measurement], config: Config) -> Result<()> {
    let ref x_axis = compute_x_axis_from_tests(measurements);
    plot_with_x_axis(measurements, &x_axis, config)
}

fn plot_indices_as_x(measurements: &[Measurement], config: Config) -> Result<()> {
    let ref x_axis = compute_x_axis_from_indices(measurements);
    plot_with_x_axis(measurements, &x_axis, config)
}

fn plot_with_x_axis(measurements: &[Measurement], x_axis: &XAxis, config: Config) -> Result<()> {
    let ref data_sets = compute_data_sets(measurements);

    let mut fg = Figure::new();

    {
        let axes = fg.axes2d();
        axes.set_x_axis(true, &[]);
        axes.set_x_label(&x_axis.axis_label, &[]);
        axes.set_y_axis(true, &[]);
        if !config.compute_normalize {
            axes.set_y_label("ns/iter", &[]);
        } else {
            axes.set_y_label("normalized ns/iter", &[]);
        }

        if let Some(ref ticks) = x_axis.ticks {
            let gnu_ticks = ticks.iter()
                .enumerate()
                .map(|(i, s)| Tick::Major(i, AutoOption::Fix(s.to_string())));
            axes.set_x_ticks_custom(gnu_ticks, &[], &[]);
        }

        for (ds_name, ds_measurements) in data_sets {
            let xs = ds_measurements.iter().map(|&i| x_axis.coords[i]);
            let ys = ds_measurements.iter().map(|&i| measurements[i].time);

            let name = escape(&ds_name);
            let options = vec![PlotOption::Caption(&name)];
            if !config.include_variance {
                axes.points(xs, ys, &options);
            } else {
                // cargo bench reports the diff between max/min. That
                // means we want a bar of equal height on top and
                // bottom, so divide by 2.
                let y_errors = ds_measurements.iter().map(|&i| measurements[i].variance);
                axes.y_error_lines(xs, ys, y_errors, &options);
            }
        }
    }

    fg.set_terminal("svg", config.output_file);
    fg.show();

    println!("plot generated to `{}`", config.output_file);
    Ok(())
}

struct XAxis {
    // how to label the X axis
    axis_label: String,

    // for each measurement, what is its x coordinate?
    coords: Vec<usize>,

    // how to label each x coordinate? (if None, use numbers)
    ticks: Option<Vec<String>>,
}

/// Compute the X axis based on the commit name. We assume that the
/// commits first appear in the order desired. The user can sort if
/// that is not the case.
fn compute_x_axis_from_commits(measurements: &[Measurement]) -> XAxis {
    compute_x_axis_from_names(measurements, "commit", |m| &m.commit)
}

/// Compute the X axis based on the test name. We assume that the
/// commits first appear in the order desired. The user can sort if
/// that is not the case.
fn compute_x_axis_from_tests(measurements: &[Measurement]) -> XAxis {
    compute_x_axis_from_names(measurements, "test", |m| &m.test)
}

fn compute_x_axis_from_names<F>(measurements: &[Measurement], axis_label: &str, name: F) -> XAxis
    where F: Fn(&Measurement) -> &String
{
    let mut set = HashSet::new();
    let ref map: HashMap<_, _> = measurements.iter()
        .map(|m| name(m))
        .filter(|&commit| set.insert(commit))
        .map(|commit| commit.to_string())
        .zip(0..)
        .collect();

    let mut ticks: Vec<_> = (0..map.len()).map(|_| String::new()).collect();
    for (commit, &index) in map {
        ticks[index] = commit.clone();
    }

    let coords: Vec<_> = measurements.iter().map(|m| map[name(m)]).collect();

    XAxis {
        axis_label: axis_label.to_string(),
        coords: coords,
        ticks: Some(ticks),
    }
}

fn compute_x_axis_from_indices(measurements: &[Measurement]) -> XAxis {
    XAxis {
        axis_label: "measurement".to_string(),
        coords: (0..measurements.len()).collect(),
        ticks: None,
    }
}

fn compute_data_sets(measurements: &[Measurement]) -> HashMap<String, Vec<usize>> {
    let mut result = HashMap::new();
    for (i, m) in measurements.iter().enumerate() {
        result.entry(m.test.clone()).or_insert(vec![]).push(i);
    }
    result
}

fn escape(name: &str) -> String {
    // GNU plot converts `_` into subscript; I can't find a way to
    // disable this escaping in the Rust wrapper so...
    name.replace('_', "-")
}

fn compute_medians(measurements: &[Measurement], normalize: bool) -> Vec<Measurement> {
    let mut keys = vec![];
    let mut map = HashMap::new();
    for measurement in measurements {
        let key = (measurement.commit.clone(), measurement.test.clone());
        let aggregated = map.entry(key.clone())
            .or_insert_with(|| {
                keys.push(key);
                vec![]
            });
        aggregated.push(measurement.time);
    }

    // sort the values
    for (_key, values) in &mut map {
        values.sort();
    }

    if !normalize {
        return keys.into_iter()
            .map(|key| {
                let (median, error) = compute_median_and_error(&map[&key]);
                Measurement {
                    commit: key.0,
                    test: key.1,
                    time: median,
                    variance: error,
                }
            })
            .collect();
    }

    let mut baselines = HashMap::new();
    for key in &keys {
        // for each test, take the first commit we found
        baselines.entry(key.1.clone())
            .or_insert_with(|| {
                let (median, _error) = compute_median_and_error(&map[key]);
                median
            });
    }

    keys.into_iter()
        .map(|key| {
            let baseline = baselines[&key.1];
            let (median, error) = compute_median_and_error(&map[&key]);
            Measurement {
                commit: key.0,
                test: key.1,
                time: scale(median, baseline),
                variance: scale(error, baseline),
            }
        })
        .collect()
}

/// Input: sorted list of u64.
/// Output: median and maximum error
fn compute_median_and_error(values: &[u64]) -> (u64, u64) {
    let len = values.len();
    let median = if len == 0 {
        0
    } else if len % 2 == 1 {
        // odd number. pick the one in the middle.
        //
        // [0, 1, 2, 3]
        values[len / 2 - 1] / 2 + values[len / 2] / 2
    } else {
        // even number: average the two in the middle.
        //
        // [0, 1, 2]
        values[len / 2]
    };

    let error = if len == 0 {
        0
    } else {
        cmp::max(median - values[0], values[len - 1] - median)
    };

    (median, error)
}

fn scale(value: u64, mut baseline: u64) -> u64 {
    if baseline == 0 {
        // Should basically never happen. It would mean test took 0ns to
        // run.
        baseline = 1;
    }
    let value = value as f64;
    let baseline = baseline as f64;
    let percent = (value / baseline) * 100.0;
    percent as u64
}
