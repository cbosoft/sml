# SML - ShakeMyLeg, is that a State Machine Language?

A simple state machine definition language and interpreter.

```sml
# flip_flip.sml

state A:
    when true:
        changeto B

state B:
    when true:
        changeto A
```


```sml
state setup:
    when i.temperature > g.low_temperature:
        changeto init_heat
    when i.temperature < g.low_temperature:
        changeto init_cool
    when i.temperature ~= g.low_temperature:
        changeto heat

state init_heat:
    on_entry:
        o.action = heat
        o.target_temperature = g.high_temperature
        o.heat_rate = g.heat_rate
    when true:
        changeto A
```
