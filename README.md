# MCP23017 RPI Library

This library provides an interface for the MCP23017 gpio expander chip. It is based on a python library of the same name, found [here](https://github.com/GillesC/MCP23017-RPI-Lib).

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

## License
This software is provided under the MIT license. Click [here](./LICENSE) to view. The original python library is also licensed under MIT. It can be found [here](./LICENSE_OLD).