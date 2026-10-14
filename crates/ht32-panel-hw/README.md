# ht32-panel-hw

Hardware abstraction library for HT32-based LCD displays and RGB LED strips found in mini PCs like the [Skullsaints Agni](https://www.electroniksindia.com/products/agni-by-skullsaints-mini-pc-intel-twin-lake-n150-vibrant-lcd-screen-m-2-ssd-mini-tower-with-rgb-lights-wifi-6-4k-uhd-dual-lan-for-home-and-office) and [AceMagic S1](https://acemagic.com/products/acemagic-s1-12th-alder-laker-n95-mini-pc).

## Supported Hardware

| Component | Interface | Details |
|-----------|-----------|---------|
| LCD Display | USB HID | VID:PID 04D9:FD01, 320x170 RGB565 |
| LED Strip | Serial | CH340, 10000 baud |

## Usage

```rust
use ht32_panel_hw::{Lcd, Led};

// LCD display
let lcd = Lcd::open()?;
lcd.set_orientation(Orientation::Landscape)?;
lcd.draw_frame(&image_data)?;

// LED strip
let led = Led::open("/dev/ttyUSB0")?;
led.set_effect(Effect::Rainbow, 3, 3)?;
```

## License

AGPL-3.0-or-later
