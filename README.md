This algorithm is the product of a collaboration between Swiss Federal Railways SBB and ETH Zürich with the goal to compute an optimized rolling stock scheduling.

The objectives are to minimize the number of rolling stock units and the travel distance of dead-head-trips, while meeting the dynamic passenger demand and still satisfying the maintenance regulations.

Our optimization approach combines local-search meta-heuristics and the classic network simplex for computing a minimum-cost circulation. Local search improves rolling stock schedules through small, local modifications, such as exchanging adjusting train compositions, or exchanging a sequence of service trips to another rolling stock vehicle. Simultaneously, the optimality of the network simplex ensures minimal overall costs.

# Deployment via Docker

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

# Server Usage

- send `POST http://localhost:3000/solve` with a JSON body containing the input. After solving the solution is returned as JSON.

- send `GET http://localhost:3000/health` to see if the server is running.

- `Insomnia` or `Postman` can send this requests with a GUI.

- or `curl`:

  ```bash
  curl -X POST -H "Content-Type: application/json" -d @path/to/input.json http://localhost:3000/solve
  ```

# Single Run

- choose the instance in internal/src/main.rs

- from the main directory, compile and run the program with:

  ```bash
  cargo run --bin=single_run --release
  ```

- limiting the number of thread:

  ```bash
  RAYON_NUM_THREADS=16 cargo run --bin=single_run --release
  ```

# Start Server (without Docker)

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

- model for an rolling stock scheduling instance

- provides base types (Ids, Distance, Cost,...), VehicleTypes, Locations, Config, Network, as well as the functionality to (de)serialize from json

- see `model/resources/small_test_input.json` for an example input

- all this data stay constant during one run

- VehicleTypes:

  - provides VehicleTypes consisting of a VehicleTypeId, name, seats, capacity, and maximal formation length

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

- a Vehicle consists of an Id and a vehicle type

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

#### solver framework (work in progress)

- a generic solver trait, that each solver should implement

- meta-heuristic local search algorithms which are independent of the rolling stock scheduling problem

#### solver (work in progress)

- min-cost-circulation algorithm via the rs_graph crate (using the network simplex)

- implements the local search meta-heuristic from the solver framework for the rolling stock scheduling problem (TODO)

- defines the objective (at the moment)

#### #### server

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

#### python_visualization

- a schedule in json-format can be visualized using poetry
- see the README.md within the python_visualization folder
