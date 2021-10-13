use crate::types::TimeSeriesPoint;

pub trait Graph {
    fn from_time_series(data : &Vec<TimeSeriesPoint>) -> Self;  
} 
pub struct CandleStickGraph;
impl Graph for CandleStickGraph {
    fn from_time_series(data : &Vec<TimeSeriesPoint>) -> CandleStickGraph {
        return CandleStickGraph{};
    }
}
