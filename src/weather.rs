use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct GeoResponse {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    current_weather: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature: f64,
    #[serde(default)]
    weathercode: i64,
}

#[derive(Debug, Clone)]
pub struct WeatherInfo {
    pub temperature_f: f64,
    pub description: String,
    pub icon: String,
}

pub fn fetch_weather() -> Option<WeatherInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build();

    // ponytail: ip-api free tier is http-only; only lat/lon cross the wire
    let geo: GeoResponse = agent
        .get("http://ip-api.com/json/")
        .call()
        .ok()?
        .into_json()
        .ok()?;

    let weather_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&temperature_unit=fahrenheit",
        geo.lat, geo.lon
    );

    let forecast: ForecastResponse = agent.get(&weather_url).call().ok()?.into_json().ok()?;

    let (description, icon) = weather_code_display(forecast.current_weather.weathercode);

    Some(WeatherInfo {
        temperature_f: forecast.current_weather.temperature,
        description: description.to_string(),
        icon: icon.to_string(),
    })
}

/// Maps an Open-Meteo WMO weather code to (description, icon).
fn weather_code_display(code: i64) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear", "\u{2600}"),
        1..=3 => ("Partly cloudy", "\u{26C5}"),
        45 | 48 => ("Foggy", "\u{1F32B}"),
        51 | 53 | 55 => ("Drizzle", "\u{1F327}"),
        61 | 63 | 65 => ("Rain", "\u{1F327}"),
        71 | 73 | 75 => ("Snow", "\u{2744}"),
        77 => ("Snow grains", "\u{2744}"),
        80..=82 => ("Showers", "\u{1F327}"),
        85 | 86 => ("Snow showers", "\u{1F328}"),
        95 => ("Thunderstorm", "\u{26C8}"),
        96 | 99 => ("Thunderstorm", "\u{26C8}"),
        _ => ("Unknown", "\u{2601}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes_map_to_descriptions() {
        assert_eq!(weather_code_display(0).0, "Clear");
        assert_eq!(weather_code_display(2).0, "Partly cloudy");
        assert_eq!(weather_code_display(63).0, "Rain");
        assert_eq!(weather_code_display(95).0, "Thunderstorm");
    }

    #[test]
    fn unknown_code_falls_back() {
        assert_eq!(weather_code_display(1234).0, "Unknown");
    }
}
