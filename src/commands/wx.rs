use chrono::prelude::*;
use plotters::{backend::BitMapBackend, drawing::IntoDrawingArea, prelude::*};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tokio::fs::File;

use crate::lib::{config, error::Error, utils, utils::GeocodeResponse};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    creationDate: chrono::DateTime<Utc>,
    pub location: LocationData,
    time: ForecastTime,
    data: ForecastData,
    currentobservation: CurrentData,
}

#[derive(Debug, Deserialize)]
pub struct LocationData {
    pub zone: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ForecastTime {
    startPeriodName: Vec<String>,
    tempLabel: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastData {
    temperature: Vec<String>,
    text: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CurrentData {
    pub Temp: String,
    pub Dewp: String,
    pub Relh: String,
    pub Winds: String,
    pub Windd: String,
    pub Gust: String,
    pub Weather: String,
    pub Visibility: String,
    pub SLP: String,
    pub WindChill: String,
}

pub async fn fetch_wx(lat: f64, lon: f64) -> Result<WeatherResponse, Error> {
    let config = config::Config::load_config()?;
    let url = format!(
        "https://forecast.weather.gov/MapClick.php?lat={lat}&lon={lon}&unit=0&lg=english&FcstType=json");
    let client = reqwest::ClientBuilder::new().user_agent(config.user_agent).build()?;
    let resp = client.get(&url).send().await?.json().await;

    match resp {
        Ok(data) => {
            let resp: WeatherResponse = data;
            Ok(resp)
        }
        Err(_) => Err(Error::NotFound("The zip code provided does not match a location".into())),
    }
}

async fn parse_current(data: GeocodeResponse) -> String {
    let (city, state, lat, lon) = (
        &data.results[0].name,
        &data.results[0].admin1,
        data.results[0].latitude,
        data.results[0].longitude,
    );

    match fetch_wx(lat, lon).await {
        Ok(data) => {
            format!(
                "```
Current Weather => {}, {} (lat: {:.2}, lon: {:.2})

Temperature:        {}
Dew:                {}
Humidity:           {}
Wind Speed:         {}
Wind Direction:     {} {}
Wind Gust:          {}
Pressure:           {}
Weather:            {}
Visibility:         {}
Wind Chill          {}
```",
                city,
                state,
                lat,
                lon,
                if data.currentobservation.Temp == "NA" || data.currentobservation.Temp.is_empty() {
                    String::from("-")
                } else {
                    let temp = data.currentobservation.Temp;
                    format!("{temp}\u{b0}")
                },
                if data.currentobservation.Dewp == "NA" || data.currentobservation.Dewp.is_empty() {
                    String::from("-")
                } else {
                    let dew = data.currentobservation.Dewp;
                    format!("{dew}\u{b0}")
                },
                if data.currentobservation.Relh == "NA" || data.currentobservation.Relh.is_empty() {
                    String::from("-")
                } else {
                    let humidity = data.currentobservation.Relh;
                    format!("{humidity}%")
                },
                if data.currentobservation.Winds == "NA" || data.currentobservation.Winds.is_empty()
                {
                    String::from("-")
                } else {
                    let wind = data.currentobservation.Winds;
                    format!("{wind} MPH")
                },
                if data.currentobservation.Windd == "NA" || data.currentobservation.Windd.is_empty()
                {
                    String::from("-")
                } else {
                    utils::cardinal_direction(&data.currentobservation.Windd)
                },
                if data.currentobservation.Windd == "NA" || data.currentobservation.Windd.is_empty()
                {
                    String::new()
                } else {
                    let wind_direction = data.currentobservation.Windd;
                    format!("({wind_direction}\u{b0})")
                },
                if data.currentobservation.Gust == "NA" || data.currentobservation.Gust.is_empty() {
                    String::from("-")
                } else {
                    data.currentobservation.Gust
                },
                // TODO: Trim trailing zeroes and decimal point
                if data.currentobservation.SLP == "NA" || data.currentobservation.SLP.is_empty() {
                    String::from("-")
                } else {
                    let alt = data.currentobservation.SLP;
                    format!("{alt} inHg")
                },
                if data.currentobservation.Weather == "NA"
                    || data.currentobservation.Weather.is_empty()
                {
                    String::from("-")
                } else {
                    data.currentobservation.Weather
                },
                if data.currentobservation.Visibility == "NA"
                    || data.currentobservation.Visibility.is_empty()
                {
                    String::from("-")
                } else {
                    let visibility = data.currentobservation.Visibility;
                    format!("{visibility} SM")
                },
                if data.currentobservation.WindChill == "NA"
                    || data.currentobservation.WindChill.is_empty()
                {
                    String::from("-")
                } else {
                    data.currentobservation.WindChill
                }
            )
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("current")]
pub async fn wx_current(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => match utils::fetch_location(zip_code).await {
                Ok(data) => {
                    let data = parse_current(data).await;
                    msg.channel_id.say(&ctx.http, data).await?
                }
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            },
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}

pub async fn parse_forecast(zip_code: i32) -> String {
    match utils::fetch_location(zip_code).await {
        Ok(data) => {
            let (city, state, lat, lon) = (
                &data.results[0].name,
                &data.results[0].admin1,
                data.results[0].latitude,
                data.results[0].longitude,
            );
            match fetch_wx(lat, lon).await {
                Ok(data) => {
                    let mut forecast = String::new();
                    let time = Local
                        .from_utc_datetime(&data.creationDate.naive_local())
                        .format("%I:%M %p");

                    for i in 0..5 {
                        forecast.push_str(&format!(
                            "\n\n{} ({}: {})\n-----------------------\n\n{}",
                            data.time.startPeriodName[i],
                            data.time.tempLabel[i].to_lowercase(),
                            data.data.temperature[i],
                            data.data.text[i]
                        ));
                    }

                    format!(
                    "```Weather Forecast => {}, {} (lat: {:.2}, lon: {:.2}) {}\n\nLast updated at {}```",
                    city, state, lat, lon, forecast, time
                )
                }
                Err(e) => format!("`There was an error retrieving data: {e}`"),
            }
        }
        Err(e) => format!("`There was an error retrieving data: {e}`"),
    }
}

#[command]
#[aliases("forecast")]
pub async fn wx_forecast(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => {
                let data = parse_forecast(zip_code).await;
                msg.channel_id.say(&ctx.http, data).await?
            }
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}

fn create_forecast_graph(
    city: &str,
    state: &str,
    label: &str,
    temps: &[i32],
) -> Result<String, Error> {
    let timestamp: DateTime<Utc> = Utc::now();
    let file_name =
        format!("./attachments/{}_forecast_graph.png", timestamp.format("%y_%m_%d_%H%M%S"));

    let min = temps.iter().min().unwrap();
    let max = temps.iter().max().unwrap();

    let (temp_highs, temp_lows) = if label.contains("Low") {
        let highs: Vec<i32> = temps.iter().skip(1).step_by(2).map(|x| x.to_owned()).collect();
        let lows: Vec<i32> = temps.iter().step_by(2).map(|x| x.to_owned()).collect();

        (highs, lows)
    } else {
        let highs: Vec<i32> = temps.iter().step_by(2).map(|x| x.to_owned()).collect();
        let lows: Vec<i32> = temps.iter().skip(1).step_by(2).map(|x| x.to_owned()).collect();

        (highs, lows)
    };

    let root_area = BitMapBackend::new(&file_name, (1024, 768)).into_drawing_area();

    root_area.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .margin(30)
        .set_label_area_size(LabelAreaPosition::Left, 64)
        .set_label_area_size(LabelAreaPosition::Bottom, 64)
        .caption(format!("Forecasted Temperatures for {city}, {state}"), ("sans-serif", 36))
        .build_cartesian_2d(0..6, (min - 10)..(max + 10))
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Day")
        .y_desc("Temperature")
        .label_style(("sans-serif", 24))
        .draw()
        .unwrap();
    chart
        .draw_series(
            LineSeries::new(
                temp_highs.iter().enumerate().map(|(i, temp)| {
                    let x = i as i32;
                    let y = *temp;
                    (x, y)
                }),
                &RED,
            )
            .point_size(2),
        )
        .unwrap()
        .label("High")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
    chart
        .draw_series(
            LineSeries::new(
                temp_lows.iter().enumerate().map(|(i, temp)| {
                    let x = i as i32;
                    let y = *temp;
                    (x, y)
                }),
                &BLUE,
            )
            .point_size(2),
        )
        .unwrap()
        .label("Low")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));
    chart
        .configure_series_labels()
        .label_font(("sans-serif", 18))
        .position(SeriesLabelPosition::MiddleRight)
        .border_style(BLACK)
        .background_style(WHITE.mix(0.8))
        .draw()
        .unwrap();

    Ok(file_name.to_string())
}

#[command]
#[aliases("graph")]
pub async fn wx_graph(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = match utils::check_location(ctx, msg, &args).await {
        Ok(val) => val,
        Err(_) => String::new(),
    };
    let args: Vec<&str> = args.split(' ').collect();

    for arg in args {
        match utils::check_zip_code(arg) {
            Ok(zip_code) => match utils::fetch_location(zip_code).await {
                Ok(data) => {
                    let (city, state, lat, lon) = (
                        &data.results[0].name,
                        &data.results[0].admin1,
                        data.results[0].latitude,
                        data.results[0].longitude,
                    );
                    match fetch_wx(lat, lon).await {
                        Ok(data) => {
                            let temps: Vec<i32> = data
                                .data
                                .temperature
                                .iter()
                                .map(|x| x.parse::<i32>().unwrap())
                                .collect();
                            let file_name = match create_forecast_graph(
                                city,
                                state,
                                &data.time.tempLabel[0],
                                &temps,
                            ) {
                                Ok(val) => val,
                                Err(e) => {
                                    msg.channel_id
                                        .say(&ctx.http, format!("`Error creating chart: {e}`"))
                                        .await?;
                                    return Ok(());
                                }
                            };
                            let file = match File::open(file_name).await {
                                Ok(f) => f,
                                Err(e) => {
                                    msg.channel_id
                                        .say(&ctx.http, format!("`Error opening image file: {e}`"))
                                        .await?;
                                    return Ok(());
                                }
                            };
                            let file = vec![(&file, "forecast_graph.png")];

                            msg.channel_id.send_files(&ctx.http, file, |m| m.content("")).await?
                        }
                        Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
                    }
                }
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            },
            Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
        };
    }

    Ok(())
}
