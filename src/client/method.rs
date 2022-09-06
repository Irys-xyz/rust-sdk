use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
pub enum Method {
    Balance = 0,
    Fund = 1,
}
