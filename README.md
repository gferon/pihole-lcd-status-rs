# Rustberry Pi Playground

In the context of my journey to learn Rust, I'm working on small tools to power my Raspberry Pi at home.

You might want to ask: "but why reimplement all of this in Rust, when everything is already available in Python?".
The answer is simple: because I can, and because :crab:.

I am reimplementing basic drivers for things such as:
* [a RGB LCD backplate from Adafruit](https://www.adafruit.com/product/1110) that uses a MCP2320 I2C to GPIO extender.
* [HT16K33, a I2C controller connected to a 8x8 bi-color LED Matrix, also from Adafruit](https://learn.adafruit.com/adafruit-led-backpack/bi-color-8x8-matrix).
* [AM2320, a I2C temperature and humidity sensor](https://akizukidenshi.com/download/ds/aosong/AM2320.pdf).

There is currently two small programs:
* `pihole-lcd-status` that will show statistics pulled from the PiHole API running on the same RaspberryPi
* `winterr` displays the temperature on a 8x8 LED matrix and sends readings to InfluxDB

## Preview

![LCD display](https://github.com/gferon/rustberrypi-playground/raw/master/lcd.jpg)