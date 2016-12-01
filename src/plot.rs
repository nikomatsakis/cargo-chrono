use data::{self, Measurement};
use errors::*;
use gnuplot::{AutoOption, AxesCommon, Figure, PlotOption, Tick};
use std::collections::{HashMap, HashSet};

pub struct Config<'c> {
    pub include_variance: bool,
    pub output_file: &'c str,
}

pub fn plot(data_file: &str, config: Config) -> Result<()> {
    let ref measurements = data::load_measurements(data_file)?;

    // We have to decide now between various ways to plot.

    // If there are multiple commits, then we want to use each commit as a point
    // on the X axis.
    if measurements[1..].iter().any(|m| m.commit != measurements[0].commit) {
        return plot_commits_as_x(measurements, config);
    }

    // If there are multiple test names, use those commits as points
    // on the X axis.
    if measurements[1..].iter().any(|m| m.test != measurements[0].test) {
        return plot_tests_as_x(measurements, config);
    }

    // Else, use individual measurements as points.
    return plot_indices_as_x(measurements, config);
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
        axes.set_y_label("ns/iter", &[]);

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
                let y_errors = ds_measurements.iter().map(|&i| measurements[i].variance / 2);
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
