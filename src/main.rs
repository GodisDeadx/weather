/* Weatherbit API key: de14d5c15d2144679092eaa48d9fc254 */
/* Cage Data API key: cc8427ef85e3460098e8430a6179b4f1 */
mod weather;

use weather::*;

use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::container::{Appearance, StyleSheet};
use iced::{
    advanced::widget::operation::Focusable,
    advanced::widget::Text,
    alignment, event, executor,
    keyboard::*,
    widget,
    widget::{
        button, checkbox, column, container, image, row, text, text_input,
        text_input::{focus, Id, State},
        Image, Row,
    },
    window,
    window::icon::from_file,
    Alignment, Application, Background, Color, Command, Length, Settings, Subscription,
};
use std::process::exit;

use iced::widget::{scrollable, Scrollable};

use iced_core::widget::Widget;

use futures::future::ok;
use futures::Future;
use std::default::Default;
use std::ops::Deref;

use iced::widget::scrollable::{Scrollbar, Scroller, StyleSheet as ScrollableStyleSheet};

use iced::theme::Container as ThemeContainer;
use iced::theme::Scrollable as ThemeScrollable;

use chrono::{prelude::*, FixedOffset, NaiveTime, Timelike, Utc};
use iced::widget::{Column, Container};
use iced_core::BorderRadius;
use window::Action::Close;

#[cfg(target_os = "windows")]
mod windows {
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::ShowWindow;
    use winapi::um::winuser::SW_HIDE;

    pub fn hide_console() {
        unsafe {
            let console_window = GetConsoleWindow();
            ShowWindow(console_window, SW_HIDE);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod windows {
    pub fn hide_console() {}
}

fn hide_console() {
    windows::hide_console();
}

#[derive(Debug, Clone)]
pub struct WeatherInfo {
    use_celsius: bool,
    location: String,
    sunrise: String,
    sunset: String,
    temperature: f64,
    app_temp: f64,
    dew_point: f64,
    weather: String,
    precip: f64,
    wind_speed: f64,
    wind_direction: String,
    air_pressure: f64,
    humidity: f64,
    uv_index: f64,
    air_quality: f64,
}

#[derive(Default)]
struct Weather {
    city: String,
    state: String,
    use_celsius: bool,
    weather: Option<WeatherInfo>,
    focus: Option<Focus>,
    theme: Theme,
}

#[derive(Debug, Clone)]
enum ThemeType {
    Custom,
}

#[derive(Debug, Clone)]
enum Message {
    CityChanged(String),
    StateChanged(String),
    CitySubmitted,
    WeatherUpdated(WeatherInfo),
    UseCelsius(bool),
    Refresh,
    TabPressed,
    Tabbed(Event),
    Theme(ThemeType),
}

enum Focus {
    City,
    State,
}

fn load() -> impl Future<Output = Result<ThemeType, ()>> {
    futures::future::ready(Ok(ThemeType::Custom))
}

fn container_theme() -> ThemeContainer {
    ThemeContainer::Custom(Box::new(ContainerTheme) as Box<dyn StyleSheet<Style = iced::Theme>>)
}

fn scrollable_theme() -> ThemeScrollable {
    ThemeScrollable::Custom(
        Box::new(ScrollableTheme) as Box<dyn ScrollableStyleSheet<Style = iced::Theme>>
    )
}

#[derive(Debug, Clone, Copy)]
struct ContainerTheme;

#[derive(Debug, Clone, Copy)]
struct ScrollableTheme;

impl StyleSheet for ContainerTheme {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        let mut appearance = Appearance {
            border_radius: BorderRadius::from(5.0),
            ..Appearance::default()
        };

        let red = 6.7 / 255.0;
        let green = 6.7 / 255.0;
        let blue = 6.7 / 255.0;

        appearance.background = Some(Background::Color(Color::from_rgb(
            red + 0.1,
            green + 0.1,
            blue + 0.1,
        )));
        appearance
    }
}

impl ScrollableStyleSheet for ScrollableTheme {
    type Style = iced::Theme;

    fn active(&self, style: &Self::Style) -> Scrollbar {
        let red = 6.7 / 255.0;
        let green = 6.7 / 255.0;
        let blue = 6.7 / 255.0;
        Scrollbar {
            background: Some(Background::Color(Color::from_rgb(red, green, blue))), // Customize the background color of the scrollbar
            border_radius: BorderRadius::from(5.0), // Customize the border radiux s of the scrollbar
            border_width: 0.0,                      // Customize the border width of the scrollbar
            border_color: Color::from_rgb(0.5, 0.5, 0.5), // Customize the border color of the scrollbar
            scroller: Scroller {
                color: Color::from_rgb(red + 0.1, green + 0.1, blue + 0.1), // Customize the color of the scroller
                border_radius: BorderRadius::from(5.0), // Customize the border radius of the scroller
                border_width: 0.0, // Customize the border width of the scroller
                border_color: Color::from_rgb(0.5, 0.5, 0.5), // Customize the border color of the scroller
            },
        }
    }

    fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> Scrollbar {
        self.active(style)
    }
}

impl Application for Weather {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let mut weather = Self::default();
        let mut use_celcius = weather.use_celsius;

        let startup_command = Command::perform(load(), move |result| {
            match result {
                Ok(theme_type) => Message::Theme(theme_type),
                Err(_) => {
                    // Handle the error case if needed
                    // For now, we'll use a default theme if loading fails
                    Message::Theme(ThemeType::Custom)
                }
            }
        });

        (weather, startup_command)
    }

    fn title(&self) -> String {
        String::from("Weather")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        let location_api_key = "cc8427ef85e3460098e8430a6179b4f1";
        let weather_api_key = "de14d5c15d2144679092eaa48d9fc254";
        match message {
            Message::CityChanged(city) => {
                self.city = city;
                Command::none()
            }
            Message::CitySubmitted => {
                if self.city.is_empty() {
                    return Command::none();
                }
                match get_coords(location_api_key, &self.city, &self.state) {
                    Ok(geometry) => {
                        match get_weather(weather_api_key, geometry.lat, geometry.lng) {
                            Ok(weather_info) => {
                                self.weather = Some(weather_info);
                                // let ss = window::minimize(true);
                                // ss
                                Command::none()
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                                exit(1);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error Translating: {}", err);
                        exit(1);
                    }
                }
            }
            Message::StateChanged(state) => {
                self.state = state;
                Command::none()
            }
            Message::WeatherUpdated(info) => {
                self.weather = Some(info);
                Command::none()
            }
            Message::UseCelsius(checked) => {
                if checked {
                    self.use_celsius = checked;
                    Command::none()
                } else if !checked {
                    self.use_celsius = checked;
                    Command::none()
                } else {
                    Command::none()
                }
            }
            Message::Refresh => {
                if self.city.is_empty() {
                    return Command::none();
                }
                match get_coords(location_api_key, &self.city, &self.state) {
                    Ok(geometry) => {
                        match get_weather(weather_api_key, geometry.lat, geometry.lng) {
                            Ok(weather_info) => {
                                self.weather = Some(weather_info);
                                Command::none()
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                                exit(1);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error Translating: {}", err);
                        exit(1);
                    }
                }
            }
            Message::Theme(theme) => {
                self.theme = match theme {
                    ThemeType::Custom => Theme::custom(theme::Palette {
                        background: Color::from_rgb(0.2, 0.2, 0.2),
                        text: Color::WHITE,
                        primary: Color::from_rgb(1.0, 0.5, 0.0),
                        success: Color::from_rgb(0.0, 1.0, 0.0),
                        danger: Color::from_rgb(1.0, 1.0, 0.0),
                    }),
                };
                Command::none()
            }
            Message::TabPressed => widget::focus_next(),
            Message::Tabbed(event) => {
                let msg = Message::TabPressed;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events_with(|event, _| {
            if let iced::Event::Keyboard(keyboard_event) = event {
                if let iced::keyboard::Event::KeyPressed { key_code, .. } = keyboard_event {
                    if key_code == iced::keyboard::KeyCode::Tab {
                        return Some(Message::TabPressed);
                    }
                }
            }
            None
        })
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let city_id = Id::new("city");
        let state_id = Id::new("state");

        let mut city = text_input("City", &self.city)
            .on_input(Message::CityChanged)
            .on_submit(Message::CitySubmitted)
            .id(city_id);

        let mut state = text_input("State", &self.state)
            .on_input(Message::StateChanged)
            .on_submit(Message::CitySubmitted)
            .id(state_id);

        let mut weather_text = String::new();

        // Seperate each item in weather_info into its own column

        let weather_text: Column<_> = match &self.weather {
            Some(weather_info) => {
                let utc_sunrise_time = NaiveTime::parse_from_str(&weather_info.sunrise, "%H:%M")
                    .expect("Failed to parse time");
                let utc_sunset_time = NaiveTime::parse_from_str(&weather_info.sunset, "%H:%M")
                    .expect("Failed to parse time");

                let est_offset =
                    FixedOffset::west_opt(5 * 60 * 60).expect("Failed to create EST offset");

                let utc_sunrise_datetime = Utc::today().and_time(utc_sunrise_time);
                let utc_sunset_datetime = Utc::today().and_time(utc_sunset_time);

                let est_sunrise_datetime = est_offset.from_utc_datetime(
                    &utc_sunrise_datetime
                        .expect("Failed to create EST datetime")
                        .naive_local(),
                );
                let est_sunset_datetime = est_offset.from_utc_datetime(
                    &utc_sunset_datetime
                        .expect("Failed to create EST datetime")
                        .naive_local(),
                );

                let est_sunrise_time = est_sunrise_datetime.format("%I:%M %p").to_string();
                let est_sunset_time = est_sunset_datetime.format("%I:%M %p").to_string();

                let sunrise_image_path = get_path("img\\sunrise.png");
                let sunset_image_path = get_path("img\\sunset.png");

                let temp_image_path = get_path("img\\temp.png");
                let dew_point_image_path = get_path("img\\dew_point.png");
                let humidity_image_path = get_path("img\\humidity.png");
                let wind_speed_image_path = get_path("img\\wind_speed.png");
                let wind_direction_image_path = get_path("img\\wind_direction.png");
                let air_quality_image_path = get_path("img\\air_quality.png");
                let air_pressure_image_path = get_path("img\\air_pressure.png");
                let precip_image_path = get_path("img\\precipitation.png");

                let cloudy_image_path = get_path("img\\cloudy.png");
                let partly_cloudy_image_path = get_path("img\\partly_cloudy.png");
                let sunny_image_path = get_path("img\\sunny.png");
                let fog_image_path = get_path("img\\foggy.png");
                let rain_image_path = get_path("img\\rain.png");
                let snow_image_path = get_path("img\\snow.png");
                let thunderstorm_image_path = get_path("img\\thunder.png");
                let scattered_clouds_image_path = get_path("img\\scattered_cloud.png");

                let weather_image_path = match weather_info.weather.as_str() {
                    "Cloudy" => cloudy_image_path,
                    "Overcast clouds" => cloudy_image_path,
                    "Broken clouds" => partly_cloudy_image_path,
                    "Scattered clouds" => scattered_clouds_image_path,
                    "Few clouds" => partly_cloudy_image_path,
                    "Sunny" => sunny_image_path,
                    "Clear sky" => sunny_image_path,
                    "Thunderstorm" => thunderstorm_image_path,
                    "Rain" => rain_image_path,
                    "Fog" => fog_image_path,
                    "Freezing fog" => fog_image_path,
                    "Haze" => fog_image_path,
                    "Drizzle" => rain_image_path,
                    "Heavy drizzle" => rain_image_path,
                    "Light drizzle" => rain_image_path,
                    "Shower rain" => rain_image_path,
                    "Heavy shower rain" => rain_image_path,
                    "Light shower snow" => snow_image_path,
                    "Snow shower" => snow_image_path,
                    "Heavy snow" => snow_image_path,
                    "Snow" => snow_image_path,
                    "Freezing rain" => rain_image_path,
                    "Flurries" => snow_image_path,
                    "Mix snow/rain" => snow_image_path,
                    "Light snow" => snow_image_path,
                    _ => fog_image_path.clone(), // Default image path if the weather condition doesn't match any of the above
                };

                let mut temperature = weather_info.temperature;
                let mut app_temp = weather_info.app_temp;
                let mut dew_point = weather_info.dew_point;
                let mut wind_speed = weather_info.wind_speed;
                let mut air_pressure = weather_info.air_pressure;
                let mut humidity = weather_info.humidity;
                let mut precip = weather_info.precip;

                if !self.use_celsius {
                    // convert to imperial system
                    temperature = (temperature * 9.0 / 5.0) + 32.0;
                    app_temp = (app_temp * 9.0 / 5.0) + 32.0;
                    dew_point = (dew_point * 9.0 / 5.0) + 32.0;
                    wind_speed = wind_speed * 1.609344;
                    humidity = humidity * 100.0;
                    precip = precip * 0.0393701;
                }

                let sunrise_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&sunrise_image_path))
                        .push(
                            Text::new(format!("Sunrise: {}", est_sunrise_time))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let sunset_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&sunset_image_path))
                        .push(
                            Text::new(format!("Sunset: {}", est_sunset_time))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut temp_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&temp_image_path))
                        .push(
                            Text::new(if self.use_celsius {
                                format!("Temperature: {:.2} °C", temperature as f32)
                            } else {
                                format!("Temperature: {:.2} °F", temperature as f32)
                            })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut weather_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&weather_image_path))
                        .push(
                            Text::new(format!("Weather: {}", &weather_info.weather))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut feels_like_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&temp_image_path))
                        .push(
                            Text::new(if self.use_celsius {
                                format!("Feels Like: {:.2} °C", app_temp as f32)
                            } else {
                                format!("Feels Like: {:.2} °F", app_temp as f32)
                            })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut dew_point_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&dew_point_image_path))
                        .push(
                            Text::new(if self.use_celsius {
                                format!("Dew Point: {:.2} °C", dew_point as f32)
                            } else {
                                format!("Dew Point: {:.2} °F", dew_point as f32)
                            })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut precip_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&precip_image_path))
                        .push(
                            Text::new(if self.use_celsius {
                                format!("Precipitation: {:.2} mm/hr", precip as f32)
                            } else {
                                format!("Precipitation: {:.2} in/hr", precip as f32)
                            })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut wind_speed_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&wind_speed_image_path))
                        .push(
                            Text::new(if self.use_celsius {
                                format!("Wind Speed: {:.2} km/h", wind_speed as f32)
                            } else {
                                format!("Wind Speed: {:.2} mph", wind_speed as f32)
                            })
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let wind_dir_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&wind_direction_image_path))
                        .push(
                            Text::new(format!("Direction: {}", weather_info.wind_direction))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let mut air_pressure_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&air_pressure_image_path))
                        .push(
                            Text::new(format!("Pressure: {} mB", air_pressure as f32))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let humidity_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&humidity_image_path))
                        .push(
                            Text::new(format!("Humidity: {}", weather_info.humidity as f32))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let air_quality_container = Container::new(
                    Row::new()
                        .spacing(10)
                        .push(Image::new(&air_quality_image_path))
                        .push(
                            Text::new(format!("Air Quality: {}", weather_info.air_quality))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .horizontal_alignment(alignment::Horizontal::Left)
                                .vertical_alignment(alignment::Vertical::Center),
                        ),
                )
                    .style(container_theme())
                    .width(300)
                    .height(50)
                    .padding(5);

                let sun_row =
                    row![sunrise_container, sunset_container, air_quality_container,].spacing(10);

                let temps_row =
                    row![temp_container, feels_like_container, dew_point_container].spacing(10);

                let weather_row =
                    row![weather_container, precip_container, humidity_container].spacing(10);

                let wind_row = row![
                    wind_speed_container,
                    wind_dir_container,
                    air_pressure_container
                ]
                    .spacing(10);

                let first_row = Column::new().spacing(10).push(sun_row).push(temps_row);

                let second_row = Column::new().spacing(10).push(weather_row).push(wind_row);

                let final_column = Column::new().spacing(10).push(first_row).push(second_row);

                let scrollable_column = Scrollable::new(final_column)
                    .width(Length::Fill)
                    .height(Length::Fill);

                let final_column = Column::new().push(Container::new(scrollable_column));

                final_column.into()
            }
            None => column![container(text("Input a Location"))].into(),
        };

        let celsius = checkbox("Use Metric", self.use_celsius, Message::UseCelsius);
        let refresh = button("Refresh").on_press(Message::Refresh);
        //let api_keys = button("API Keys").on_press(Message::OpenApiKeys);

        let event = column![
            column![row![
                container(city).padding(10).width(250),
                container(state).padding(10).width(250),
                row![
                    container(celsius).padding(10).width(150),
                    container(refresh).padding(10).width(100).height(50),
                ]
                .spacing(50),
            ]
            .spacing(10),],
            container(weather_text).width(Length::Fill).padding(5),
        ]
            .padding(10);

        let event = Container::new(
            Scrollable::new(event)
                .style(scrollable_theme())
                .width(Length::Fill)
                .height(Length::Fill),
        )
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding(10);

        event.into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn main() {
    hide_console();

    let icon_path = get_path("img\\thunder.png");
    let icon = from_file(icon_path);
    let icon = match icon {
        Ok(icon) => icon,
        Err(error) => {
            eprintln!("Failed to create icon: {}", error);
            return;
        }
    };

    let settings = Settings {
        window: window::Settings {
            size: (960, 335),
            icon: Some(icon),
            resizable: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let _ = Weather::run(settings);
}
