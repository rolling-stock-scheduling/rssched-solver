from typing import List, Optional
from pydantic import BaseModel
from enum import Enum
from datetime import datetime


class TripType(Enum):
    SERVICE = "ServiceTrip"
    DEADHEAD = "DeadHeadTrip"


class ObjectiveValue(BaseModel):
    number_of_unserved_passengers: int
    number_of_vehicles: int
    seat_distance_traveled: int


class Trip(BaseModel):
    id: Optional[str]
    type: TripType
    origin: str
    destination: str
    departure_time: datetime
    arrival_time: datetime


class ScheduleItem(BaseModel):
    vehicle_type: str
    start_depot: str
    end_depot: str
    trips: List[Trip]


class Response(BaseModel):
    objective_value: ObjectiveValue
    schedule: List[ScheduleItem]