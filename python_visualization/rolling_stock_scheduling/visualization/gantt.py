import plotly.figure_factory as ff

from rolling_stock_scheduling.model.response import Response

CHART_COLORS = {"ServiceTrip": "rgb(220, 0, 0)", "DeadHeadTrip": (1, 0.9, 0.16)}


def respone_to_gantt(response: Response, chart_title: str):
    df = [
        {
            "Task": item.vehicle_id,
            "Start": trip.arrival_time,
            "Finish": trip.departure_time,
            "Type": trip.type.value,
        }
        for item in response.schedule
        for trip in item.trips
    ]
    fig = ff.create_gantt(
        df,
        title=chart_title,
        colors=CHART_COLORS,
        index_col="Type",
        show_colorbar=True,
        group_tasks=True,
        showgrid_x=True,
    )
    return fig
