use data::{self, Measurement};
use errors::*;
use gnuplot::{Figure, PlotOption};
use std::collections::{HashMap, HashSet};

pub fn plot(data_file: &str, include_variance: bool, output_file: &str) -> Result<()> {
    let ref measurements = data::load_measurements(data_file)?;
    let ref x_coords = compute_x_coords(measurements);
    let ref data_sets = compute_data_sets(measurements);
    let ref chars = data_set_characters(measurements);

    let mut fg = Figure::new();

    {
        let axes = fg.axes2d();
        axes.set_x_axis(true, &[]);
        axes.set_y_axis(true, &[]);
        for (ds_name, ds_measurements) in data_sets {
            let xs = ds_measurements.iter().map(|m| x_coords[&m.commit]);
            let ys = ds_measurements.iter().map(|m| m.time);

            let options = vec![PlotOption::Caption(&ds_name)];
            if !include_variance {
                axes.points(xs, ys, &options);
            } else {
                // cargo bench reports the diff between max/min. That
                // means we want a bar of equal height on top and
                // bottom, so divide by 2.
                let y_errors = ds_measurements.iter().map(|m| m.variance / 2);
                axes.y_error_lines(xs, ys, y_errors, &options);
            }
        }
    }

    fg.set_terminal("svg", output_file);
    fg.show();
    Ok(())
}

/// We compute the X axis based on the commit. We assume that the
/// commits first appear in the order desired. The user can sort if
/// that is not the case.
fn compute_x_coords(measurements: &[Measurement]) -> HashMap<String, u64> {
    let mut set = HashSet::new();
    measurements.iter()
        .map(|m| &m.commit)
        .filter(|&commit| set.insert(commit))
        .map(|commit| commit.to_string())
        .zip(0_u64..)
        .collect()
}

fn compute_data_sets(measurements: &[Measurement]) -> HashMap<String, Vec<&Measurement>> {
    let mut result = HashMap::new();
    for m in measurements {
        result.entry(m.name.clone()).or_insert(vec![]).push(m);
    }
    result
}

fn data_set_characters(measurements: &[Measurement]) -> HashMap<String, char> {
    let mut result = HashMap::new();
    let mut chars_taken = HashSet::new();
    for m in measurements {
        if result.contains_key(&m.name) {
            continue;
        }

        // pick the first character that is not taken; if we can't
        // find one, then just pick the first character (oh well)
        let first_char_not_taken = m.name.chars().filter(|&c| chars_taken.insert(c)).next();
        if let Some(ch) = first_char_not_taken {
            result.insert(m.name.clone(), ch);
        } else if let Some(ch) = m.name.chars().next() {
            result.insert(m.name.clone(), ch);
        } else {
            // use 'x' for empty string, which we really don't expect anyway
            result.insert(m.name.clone(), 'x');
        }
    }
    result
}
