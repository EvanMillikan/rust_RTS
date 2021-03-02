# Real Time System Module

## To Run
- `Cargo run` for the final iteration
- `Cargo run --example iteration1` for the first iteration

## Scenario
- The scenario of this project is an airport traffic control (ATC) system where the ATC has to take care of incoming and departing airplane. Once in a while, a storm will come and cause the runway to be unusable thus the ATC needs to call the cleaner to clean the runway so that normal task can resume. 
- The scenario will generate 60 airplanes before terminating.

## Performance Measured
1. Duration it takes for ATC to notice from when event has been generated true
2. Duration it takes for cleaner to be dispatched from when event has been generated true
3. Duration it takes for cleaner to be signalled by ATC thread from when ATC thread notice an event
4. Duration it takes for cleaner to signal ATC thread to resume normal operation

## Success Criteria
The success criteria of the project is if the 2nd + 4th performance measured combined is less than 1 secs. 

## Created By 
- **Evan Millikan TP049195**
