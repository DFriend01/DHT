use chrono::Utc;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

pub fn init_logger(level: LevelFilter, server_id: u32) {
    // Pattern
    let pattern: String = format!(
        "[{{d(%Y-%m-%d %H:%M:%S %Z)(utc)}} - {{l}} - N{}] {{m}}{{n}}",
        server_id
    );
    let pattern_colored = "{h(".to_owned() + &pattern + ")}";
    let date_now: String = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    // Appenders
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&pattern_colored)))
        .build();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&pattern)))
        .build(format!("log/N{}_{}.log", server_id, date_now))
        .unwrap();

    // Initialize the loggers
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("stdout").appender("logfile").build(level))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}
