from datetime import datetime
from enum import Enum
from typing import List, Optional

from pydantic import BaseModel


class TripType(Enum):
    SERVICE = "ServiceTrip"
    DEADHEAD = "DeadHeadTrip"


class Trip(BaseModel):
    id: Optional[str]
    type: TripType
    origin: str
    destination: str
    departure_time: datetime
    arrival_time: datetime


class ScheduleItem(BaseModel):
    vehicle_id: str
    vehicle_type: str
    start_depot: str
    end_depot: str
    trips: List[Trip]


class Response(BaseModel):
    schedule: List[ScheduleItem]
