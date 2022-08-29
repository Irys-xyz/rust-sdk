use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
pub enum Method {
    Help = 0,
    Balance = 1,
    Withdraw = 2,
    Upload = 3,
    UploadDir = 4,
    Deploy = 5,
    Fund = 6,
    Price = 7,
}
