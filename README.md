A tool for gathering and analyzing benchmark data. The idea is that
you should create a series of commits representing various changes
whose effects you would like to measure. You can run `cargo-chrono
bench` for each one and it will run `cargo bench` and accumulate the
results into a CSV file. Then you can run `cargo-chrono plot` to see
the results plotted and hence get a feeling for the effect of each
commit.

Still very early and hacky, but very useful! Note that the plotting
feature requires gnuplot to be installed.
