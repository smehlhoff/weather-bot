# weather-bot

A personal discord bot that provides METAR, TAF, ultraviolet, and current weather observations.

This bot will also send a daily ultraviolet forecast to specified users at 8 am.

## Install

Rename `config-example.json` to `config.json` and edit fields accordingly.

    $ git clone https://github.com/smehlhoff/weather-bot.git
    $ cd weather-bot
    $ cargo build --release
    $ nohup ./target/release/weather-bot &

## Usage

This bot supports the following commands:

    Return current weather observation      !wx <zip code>
    Return current METAR report             !metar <station code>
    Return current TAF report               !taf <station code>
    Return current UV index                 !uv current <zip code>
    Return forecasted UV index              !uv forecast <zip code>
    Return bot uptime                       !uptime
    This help menu                          !help

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](https://github.com/smehlhoff/weather-bot/blob/master/LICENSE)