# mercury
A (partial) rust port of Home Sensor Network 

Uses diesel, rocket, reqwest, yaml-rust and rust nightly. Currently provides only the API for the sensors.
For the full webinterface see: https://github.com/zeratul2099/home_sensor_network

Home Sensor Network / Mercury provides a database backend for self-made Arduino/ESP32-based Sensor Units for temperature
and air humidity surveillance and storing. The ESP32-code and circuit-diagram will will follow later here.

TODO: port all hsn-features
    - webgui with templates (tera as jinja2-replacement)
    - Timezones
    - plots with plotly.js
    - wouldbe-values
    - daily mean
    - weather forecast
    - serial-receiver for old arduino sensors
