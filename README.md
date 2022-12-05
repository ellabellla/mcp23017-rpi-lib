# MCP23017 RPI Library

This library provides an interface for the MCP23017 gpio expander chip. It is based on a python library of the same name, found [here](https://github.com/GillesC/MCP23017-RPI-Lib).

You can view the docs [here](https://ellabellla.github.io/mcp23017-rpi-lib/mcp23017_rpi_lib/).

## Example - Blink
```rust
let mut mcp = MCP23017::new(0x20, 1).unwrap();
let led = Pin::new(0).unwrap();
mcp.pin_mode(&led, Mode::Output).unwrap();

loop {
    mcp.output(&led, State::High).unwrap();
    sleep(Duration::from_millis(200));
    mcp.output(&led, State::Low).unwrap();
    sleep(Duration::from_millis(200));
}  
```

## Pin Conversion

| PIN | MCP PIN | Name |
| --- | ------- | ---- |
| 0   | 21      | GPA0 |
| 1   | 22      | GPA1 |
| 2   | 23      | GPA2 |
| 3   | 24      | GPA3 |
| 4   | 25      | GPA4 |
| 5   | 26      | GPA5 |
| 6   | 27      | GPA6 |
| 7   | 28      | GPA7 |
| 8   | 1       | GPB0 |
| 9   | 2       | GPB1 |
| 10  | 3       | GPB2 |
| 11  | 4       | GPB3 |
| 12  | 5       | GPB4 |
| 13  | 6       | GPB5 |
| 14  | 7       | GPB6 |
| 15  | 8       | GPB7 |

## License
This software is provided under the MIT license. Click [here](./LICENSE) to view. The original python library is also licensed under MIT. It can be found [here](./LICENSE_OLD).