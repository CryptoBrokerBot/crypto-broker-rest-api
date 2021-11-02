use crate::types::*;
use plotters::prelude::*;
use crate::types::Numeric;
use rust_decimal::prelude::*;
use crate::api::types::*;

#[allow(dead_code)]
const RGB_BYTES_PER_PIXEL : u32 = 3;

pub struct CoinPerformanceGraph {
    buffer : String
}


impl CoinPerformanceGraph {
    
    pub fn from_candle_sticks(data : &Vec<CandleStickData>, options : &GraphGenerationOptions) -> CoinPerformanceGraph {
        if data.len() == 0 {panic!("No data to graph");}

        // let mut buff = vec![0u8;(options.width * options.height * RGB_BYTES_PER_PIXEL) as usize ];
        // println!("len of buff {:?}",buff.len());
        // println!("len of buff after fill {:?}",buff.len());
        let mut string_buff = String::new();
        
        {
            // SVGBackend::with_string(buf: &'a mut String, size: (u32, u32))
            // let root = BitMapBackend::new("abcd.png", (options.width, options.heigt)).into_drawing_area();
            let root = SVGBackend::with_string(&mut string_buff, (options.width, options.height)).into_drawing_area();
            root.fill(&WHITE).expect("Could not fill background");
            let (open_date_time,close_date_time) = (
                options.from,
                options.to
            );
            let (min_low,max_high) : (f64,f64) = (
                as_f64(&data.iter().map(|d| d.low).min().expect("Could not get min")),
                as_f64(&data.iter().map(|d| d.high).max().expect("Could not get max"))
            );

            let mut chart = ChartBuilder::on(&root)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .caption(&options.caption.clone(), ("sans-serif",50.0).into_font())
                .build_cartesian_2d(open_date_time..close_date_time, min_low..max_high).expect("Could not create chart");
            chart.configure_mesh().light_line_style(&WHITE).draw().expect("Could not draw!");
            chart.draw_series(data.iter().map(|d| CandleStick::new(d.open_date_time,as_f64(&d.open),as_f64(&d.high),as_f64(&d.low),as_f64(&d.close),&GREEN,&RED,15))).expect("Could not draw series");
            root.present().expect("Could not draw to buffer");
        }
        return CoinPerformanceGraph {buffer : string_buff};
    }
}

fn as_f64(decimal : &Numeric) -> f64 {
    return decimal.to_f64().expect("Could not convert to f64");
}

use actix_web::HttpResponse;
impl Into<HttpResponse> for CoinPerformanceGraph {
    fn into(self) -> HttpResponse {
        return HttpResponse::Ok().content_type("image/svg+xml").body(self.buffer);
    }
}