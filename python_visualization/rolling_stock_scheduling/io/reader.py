import json
from datetime import datetime
from pathlib import Path
from typing import List

from rolling_stock_scheduling.model.response import (
    Response,
    ScheduleItem,
    Trip,
    TripType,
)


def parse_datetime(dt_str: str):
    return datetime.fromisoformat(dt_str)


def import_response(file_path: Path) -> Response:
    with open(file_path, "r", encoding="utf-8") as file:
        data = json.load(file)

    return Response(
        schedule=[
            ScheduleItem(
                vehicle_id=item["vehicleId"],
                vehicle_type=item["vehicleType"],
                start_depot=item["startDepot"],
                end_depot=item["endDepot"],
                trips=create_tour(item["tour"]),
            )
            for item in data["schedule"]
        ],
    )


def create_tour(tour_data) -> List[Trip]:
    trips = []
    for item in tour_data:
        for key, value in item.items():
            trip_type = (
                TripType.SERVICE if "service" in key.lower() else TripType.DEADHEAD
            )
            trip = Trip(
                id=value.get("id"),
                type=trip_type,
                origin=value["origin"],
                destination=value["destination"],
                departure_time=parse_datetime(value["departure_time"]),
                arrival_time=parse_datetime(value["arrival_time"]),
            )
            trips.append(trip)
    return trips
