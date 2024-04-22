# Immutable Datastructures Test

## Test environment

version: 1.0
instance: M332017.7.scheduler_request.json

## std:collections:HashMap + std:collections:HashMap + Vec

Minimizer: 88 sec
TakeFirst: 13.8; 12.6; 12.3 sec ~= 12.9
TakeAny: 10.0; 6.8; 9.4; 12.1; 5.7 sec ~= 8.8

## im:HashMap + im:HashSet + Vec

Minimizer: 60 sec (-32 %)
TakeFirst: 10.7; 9.3; 9.6 sec ~= 9.9 (-23 %)
TakeAny: 8.9; 1.6; 5.7; 6.8; 4.5 sec ~= 5.5 (-38 %)

## im:HashMap + im:HashSet + im:Vector

Minimizer: 60 (-32 %)
TakeFirst: 10.9; 10.4; 10.2  ~= 10.5 (-17 %)
TakeAny: 3.9; 4.1; 7.4; 9.1; 6.4 ~= 6.2 (-30 %)



# Runtime tests
Viv212, Iterations after 100 sec:

135
