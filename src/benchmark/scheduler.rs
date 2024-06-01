use colored::*;

use super::benchmark_result::BenchmarkResult;
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::{relative_speed, Benchmark};

use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options, OutputStyleOption, SortOrder};

use anyhow::Result;

pub struct Scheduler<'a> {
    commands: &'a Commands<'a>,
    options: &'a Options,
    export_manager: &'a ExportManager,
    results: Vec<BenchmarkResult>,
}

impl<'a> Scheduler<'a> {
    pub fn new(
        commands: &'a Commands,
        options: &'a Options,
        export_manager: &'a ExportManager,
    ) -> Self {
        Self {
            commands,
            options,
            export_manager,
            results: vec![],
        }
    }

    pub fn run_benchmarks(&mut self) -> Result<()> {
        let mut executor: Box<dyn Executor> = match self.options.executor_kind {
            ExecutorKind::Raw => Box::new(RawExecutor::new(self.options)),
            ExecutorKind::Mock(ref shell) => Box::new(MockExecutor::new(shell.clone())),
            ExecutorKind::Shell(ref shell) => Box::new(ShellExecutor::new(shell, self.options)),
        };

        executor.calibrate()?;

        for (number, cmd) in self.commands.iter().enumerate() {
            self.results
                .push(Benchmark::new(number, cmd, self.options, &*executor).run()?);

            // We export results after each individual benchmark, because
            // we would risk losing them if a later benchmark fails.
            self.export_manager.write_results(
                &self.results,
                self.options.sort_order_exports,
                true,
            )?;
        }

        Ok(())
    }

    pub fn print_relative_speed_comparison(&self) {
        if self.options.output_style == OutputStyleOption::Disabled {
            return;
        }

        if self.results.len() < 2 {
            return;
        }

        if let Some(annotated_results) = relative_speed::compute_with_check(
            &self.results,
            self.options.sort_order_speed_comparison,
        ) {
            match self.options.sort_order_speed_comparison {
                SortOrder::MeanTime => {
                    println!("{}", "Summary".bold());

                    let fastest = annotated_results.iter().find(|r| r.is_fastest).unwrap();
                    let others = annotated_results.iter().filter(|r| !r.is_fastest);

                    println!(
                        "  {} ran",
                        fastest.result.command_with_unused_parameters.cyan()
                    );

                    for item in others {
                        println!(
                            "{}{} times faster than {}",
                            format!("{:8.2}", item.relative_speed).bold().green(),
                            if let Some(stddev) = item.relative_speed_stddev {
                                format!(" ± {}", format!("{stddev:.2}").green())
                            } else {
                                "".into()
                            },
                            &item.result.command_with_unused_parameters.magenta()
                        );
                    }
                }
                SortOrder::Command => {
                    println!("{}", "Relative speed comparison".bold());

                    for item in annotated_results {
                        println!(
                            "  {}{}  {}",
                            format!("{:10.2}", item.relative_speed).bold().green(),
                            if item.is_fastest {
                                "        ".into()
                            } else if let Some(stddev) = item.relative_speed_stddev {
                                format!(" ± {}", format!("{stddev:5.2}").green())
                            } else {
                                "        ".into()
                            },
                            &item.result.command_with_unused_parameters,
                        );
                    }
                }
            }
        } else {
            eprintln!(
                "{}: The benchmark comparison could not be computed as some benchmark times are zero. \
                 This could be caused by background interference during the initial calibration phase \
                 of hyperfine, in combination with very fast commands (faster than a few milliseconds). \
                 Try to re-run the benchmark on a quiet system. If you did not do so already, try the \
                 --shell=none/-N option. If it does not help either, you command is most likely too fast \
                 to be accurately benchmarked by hyperfine.",
                 "Note".bold().red()
            );
        }
    }

    pub fn final_export(&self) -> Result<()> {
        self.export_manager
            .write_results(&self.results, self.options.sort_order_exports, false)
    }
}
