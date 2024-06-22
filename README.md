This algorithm is the product of a collaboration between Swiss Federal Railways SBB and ETH Zürich with the goal to compute an optimized rolling stock scheduling.

The objectives are to minimize the number of rolling stock units and the travel distance of dead-head-trips, while meeting the dynamic passenger demand and still satisfying the maintenance regulations.

Our optimization approach combines local-search meta-heuristics and the classic network simplex for computing a minimum-cost circulation. Local search improves rolling stock schedules through small, local modifications, such as exchanging adjusting train compositions, or exchanging a sequence of service trips to another rolling stock vehicle. Simultaneously, the optimality of the network simplex ensures minimal overall costs.

![local_search_modification](https://github.com/rolling-stock-scheduling/rssched-solver/assets/71029482/ebd70cde-4b51-4f18-9c8a-9f7d2a127fbc)

# Usage

In the following, we describe how to use the solver-server with or without docker or how to solve a single instance.

To get started with the whole RSSched project, have a look at this [step-by-step instruction](https://github.com/rolling-stock-scheduling/.github/blob/main/getting_started.md).

## Deployment via Docker

- install the docker engine: https://docs.docker.com/engine/install/

- building the docker image (from the main directory):

  ```bash
  docker build --tag eth_scheduling_image .
  ```

- loading the image and running the server for the first time (removes old container of the same name):

  ```bash
  docker run --rm --env RAYON_NUM_THREADS=16 --publish 3000:3000 --name eth_scheduling_server eth_scheduling_image
  ```

- the server can use 16 threads and answers on port 3000.

- if the environment variable `RAYON_NUM_THREADS` is not set, the server will use as many threads as possible.

- short version (with a random name for the container):

  ```bash
  docker run -e RAYON_NUM_THREADS=16 -p 3000:3000 eth_scheduling_image
  ```

- stopping the docker container:

  ```bash
  docker stop eth_scheduling_server
  ```

- starting it again with

  ```bash
  docker start --attach eth_scheduling_server
  ```

- remove the container:

  ```bash
  docker container rm eth_scheduling_server
  ```

## Server Usage

- send `POST http://localhost:3000/solve` with a JSON body containing the input. After solving the solution is returned as JSON.

- send `GET http://localhost:3000/health` to see if the server is running.

- `Insomnia`, `Postman`, or `Bruno` can send this requests with a GUI.

- or `curl`:

  ```bash
  curl -X POST -H "Content-Type: application/json" -d @your/input.json http://localhost:3000/solve
  ```

## Single Run

- from the main directory, compile and run the program with:

  ```bash
  cargo run --bin=single_run --release -- your/input_file.json
  ```

- limiting the number of thread:

  ```bash
    RAYON_NUM_THREADS=16 cargo run --bin=single_run --release -- your/input_file.json
  ```

## Start Server (without Docker)

- for the default port of 3000:

  ```bash
  cargo run --bin=server --release
  ```

- limiting the number of thread:

  ```bash
  RAYON_NUM_THREADS=16 cargo run --bin=server --release
  ```

- for customized port:

  ```bash
  cargo run --bin=server --release -- 4000
  ```

# Input format

The following JSON structure is used to describe the rolling stock scheduling instance. The input is a JSON object with
the following fields:

```
{
  "vehicleTypes" : [
    {
      "id" : String,
      "capacity" : Int,  // seats + standing
      "seats" : Int,
      "maximalFormationCount" : Optional[Int] // maximal number of vehicle in one formation, None means unbounded
    },
      ...
  ],
  "locations" : [
    {
      "id" : String, // e.g. Operation Point Abbreviation
    },
      ...
  ],
  "depots" : [ // Optional, if not present: all locations are depots with unlimited capacity for all vehicle types
    {
      "id" : String,
      "location" : Int,
      "capacity" : Int,  // Total capacity at depot; limits the number of vehicles at the start (and end) of the schedule.
      "allowedTypes" : [  // vehicleTypes not present are assumed to have a capacity of 0
        {
          "vehicleType" : Int,
          "capacity" : Optional[Int]  // Unbounded if not present
        },
        ...
      ]
    },
    ...
  ],
  "routes" : [
    {
       "id" : String,
       "vehicleType": String
       "segments": [
         {
           "id": String,
           "order": Int, // 0,1,2,3,...
           "origin" : String,
           "destination" : String, // origin of segment i+1 must be destination of segment i
           "distance" : Int,
           "duration" : Int,
           "maximalFormationCount" : Optional[Int]
         },
         ...
       ]
    },
    ...
  ],
  "departures" : [
    {
      "id" : String,
      "route" : String,
      "segments": [
        {
          "id": String,
          "routeSegment": String
          "departure" : DateTimeString,  // it is assumed that a vehicle can serve all segments in order, even with shunting between segments.
          "passengers" : Int,
          "seated": Int
        },
        ...
      ]
    },
    ...
  ],
  "maintenanceSlots" : [ // Optional, if not present maintenance is not considered
     {
       "id": String,
       "location": String,
       "start": DateTimeString,
       "end": DateTimeString,
       "trackCount": Int,
     },
     ...
  ],
  "deadHeadTrips" : {
    "indices" : [ String, String, ... ],  // n indices, maps Locations to index. The first location corresponds to the first row/column of the matrix
    "durations" : [ [ Int, Int, ... ], ..., [ Int, Int, ... ] ],  // n x n matrix
    "distances" : [ [ Int, Int, ... ], ..., [ Int, Int, ... ] ]  // n x n matrix
  },
  "parameters" : {
    "forbidDeadHeadTrips" : Optional[Boolean] // default is false, which means DeadHeadTrips are allowed.
    "shunting" : {
      "minimalDuration" : Int,  // minimum time that is always needed between two activities
      "deadHeadTripDuration" : Int  // change from serviceTrip to DeadHeadTrip
    },
    "maintenance" : { // optional, if not present maximalDistance is set to 0 which disables maintenance
      "maximalDistance" : Int
    }
    "costs" : { // Costs are always per second
      "staff" : Int, // each train formation on a service trip has to pay this per minute (not for dead-head-trips / idle / maintenance)
      "serviceTrip" : Int // train formation with k vehicles has to pay this k times per minute on a service trip
      "maintenance" : Optional[Int],
      "deadHeadTrip" : Int, // costs for dead head trip include the staff costs (to priotize hitch-hiking on serviceTrips the deadHeadTripCosts should be at least staff + serviceTrip
      "idle" : Int
    }
  }
}
```

For an example input see [`model/resources/small_test_input.json`](model/resources/small_test_input.json).

# Output format

The following JSON structure is used to describe a rolling stock schedule. The output is a JSON object with the
following fields:

```
{
    "info": {
        "runningTime": String // e.g. "0.01s",
        "numberOfThreads": Int,
        "timestamp(UTC)": String // e.g. "2024-04-12T07:58:12",
        "hostname": String
    },
    "objectiveValue": {
        "unservedPassengers": Int,
        "maintenanceViolation": Int,
        "vehicleCount": Int,
        "costs": Int
    },
    "schedule": {
        "depotLoads": [
            {
                "depot": String,
                "load": [
                    {
                        "vehicleType": String,
                        "spawnCount": Int
                    },
                    ...
                ]
            },
            ...
        ],


        // Vehicle perspective:
        "fleet" : [
            {
                "vehicleType": String,
                "vehicles": [
                {
                    "id": String, // new vehicleId (not present in input)
                    "startDepot": String,
                    "endDepot": String,
                    "departureSegments": [
                        {
                            "departureSegment": String
                            "origin": String,
                            "destination": String,
                            "departure": DateTimeString,
                            "arrival": DateTimeString
                        },
                        ...
                    ],
                    "maintenanceSlots": [ // only if given in input
                        {
                            "maintenanceSlot": String,
                            "location": String,
                            "start": DateTimeString,
                            "end": DateTimeString
                        },
                        ...
                    ],
                 "deadHeadTrips": [
                        {
                            "id": String // new deadHeadTripId (not present in input)
                            "origin": String,
                            "destination": String,
                            "departure": DateTimeString,
                            "arrival": DateTimeString
                        },
                        ...
                    ]

                },
                ...
            ],
            "vehicleCycles": [ // contains all vehicles in multiple cycles. Vehicle at position i of one cycle becomes vehicle at position i+1 on the same cycle for the next day (last vehicle becomes first vehicle)
                [String, String, String, ...], // each list stands for one directed cycle in the rotation graph.
                [String, ...],
                ...
            ],  // only if maintenance slots are given in input
        }


        // Trip perspective with train formations
        "departureSegments": [
            {
                "departureSegment": String,
                "origin": String,
                "destination": String,
                "departure": DateTimeString,
                "arrival": DateTimeString,
                "vehicleType": String,
                "formation": [String, String, ...], // first vehicle in at front, last vehicle at tail
            },
            ...
        ],
        "maintenanceSlots": [  // only if given in input
            {
                "maintenanceSlot": String,
                "location": String,
                "start": DateTimeString,
                "end": DateTimeString
                "formation": [String, String, ...], // first vehicle goes to the first maintenance line, etc.
            },
            ...
        ],
        "deadHeadTrips": [
            {
                "id": String,
                "origin": String,
                "destination": String,
                "departure": DateTimeString,
                "arrival": DateTimeString
                "formation": [String, String, ...], // first vehicle in at front, last vehicle at tail
            },
            ...
        ],
    }
}
```

# Development

- install the rust compiler rustc and the rust package manager cargo via rustup: https://www.rust-lang.org/tools/install

## Project Structure

The project is structured into the following sub-projects:

### time

- provides the types DateTime and Duration

- DateTime:

  - represents a point in time including year, month, day, hour, minute, second

  - enriched by Earliest (- infinity) and Latest (+ infinity)

  - integer based, whole seconds is the smallest unit

- Duration:

  - represents non-negative time duration represented by hours, minutes, seconds

  - enriched by Infinity

- basic calculations:

  - DateTime - DateTime = Duration

  - DateTime + Duration = DateTime

  - ...

### model

- model for a rolling stock scheduling instance

- provides base types (Idx, Distance, Cost,...), VehicleTypes, Locations, Config, Network, as well as the functionality to (de)serialize from json

- see `model/resources/small_test_input.json` for an example input

- all this data stay constant during one run

- VehicleTypes:

  - provides VehicleTypes consisting of a VehicleTypeIdx, name, seats, capacity, and maximal formation length

- Locations:

  - stores the locations (name, optional day limit)

  - provides dead-head-trip information between two locations (distance and duration)

- Config:

  - stores additional instance parameters, e.g., shunting durations, maintenance limits, costs coefficients

- Network:

  - stores all nodes (service trips, maintenance slots, start and end depots)

  - provides connection between these nodes via can_reach(), predecessor(), successor()

#### solution

- defines a cyclic rolling stock Schedule consisting of

  - vehicles that are used (specified by their type)

  - for each vehicle the tour it is driving

  - for each node the train formation (coupled vehicles)

  - the next day mapping, describing which vehicle of day 1 becomes which vehicle of day 2, when the schedule is repeated

  - service trips nodes that are not fully covered (= not all passengers are satisfied) are organized in dummy tours

  - note that schedules do not store their objective value

- each vehicle drives a Tour, which consists of

  - the nodes it is driving

  - some stored information for fast calculations (e.g., the total dead head distance of the tour or the cost for the tour)

  - tours always start at a start_depot_node and end at a end_depot_node

  - in between are only service trips and maintenance nodes are allowed

  - for each consecutive nodes n1 and n2 of a tour n1 can reach n2 (see Network)

- a Path is a sequence of nodes, such that consecutive nodes can be reached, but it must not start nor end at a depot node

- a Segment is a pair of a start and a end node and represents a sub path of a tour

- TrainFormation is a ordered list of vehicles, index 0 is supposed to be the front of the formation

- a Vehicle consists of an Idx and a vehicle type

- Schedules can be serialized into Json objects which are the primary part of the algorithm's output

- tour modifications:

  - tours should be immutable, so each modification creates a modified copy

  - replace start/end depot

  - remove segment

  - insert path (and remove conflicting nodes)

- schedule modifications:

  - schedules should be immutable, so each modification creates a modified copy

  - spawn vehicles (given a path or a dummy tour)

  - delete a vehicle (replace it by a dummy tour)

  - add a path to the tour of a vehicle

  - fit_reassign: given a provider and a receiver vehicle as well as a segment of the provider's tour: tries to insert as many nodes of the segment to the receiver's tour without causing any conflicts

  - override_reassign: given a provider and a receiver vehicle as well as a segment of the provider's tour: insert the segment into the receiver's tour removing all conflicting nodes

#### objective framework

- an objective consists of a hierarchy of linear combinations (levels) of indicators of a schedule (called solution)

- to define a new objective first define all indicators for a given schedule (e.g. number of unserved passenger, total dead head distance, ...) by implementing the Indicator<Schedule> trait, which needs an evaluate and a name function

- the evaluate method must return a BaseValue, which could be Integer, Float or Duration

- linear combine multiple indicators to a level by choosing Coefficients (integer or float)

- each level must be either Integer, Float or Duration and the indicators cannot be mixed within the same level

- multiple levels form an hierarchical Objective, the first level is the most important one, ties are broken by the second level and so forth

- given an Objective instance, a solution (schedule) can be evaluated. This method consumes the schedule and returns an EvaluatedSolution object which consists of the schedule and an ObjectiveValue

- an ObjectiveValue is a Vector of BaseValues which matches the objective hierarchy and implements the Ord trait.

#### heuristic framework (work in progress)

- a generic solver trait, that each meta-heuristic-solver should implement

- local search framework (independent of the rolling stock scheduling problem)

  - there are three local improver implementations which can be chosen

#### solver

- min-cost-circulation algorithm via the rs_graph crate (using the network simplex)

- implementation of the local search meta-heuristic from the heuristics framework for the rolling stock scheduling problem

  - defines the neighborhood

  - initializes the local improver

  - defines the objective

#### server

- a simple HTTP-server using the create axum.

- there are two routes /health and /solve

- /health (GET) returns "Healthy"

- /solve (POST)

  - expects a valid rolling stock scheduling instance in json form in the body (see ```model/resources/small_test_input.json``` for an example input)

  - executes the solver to produce a good schedule

  - answers with the specified output json, containing the objective value, the final schedule, as well as some additional information (running time, number of theads, timestamp, hostname)

#### internal

- this is a playground for the developer

- has a main which is similar to the /solve function of the server

- here is the place to try test-objectives
