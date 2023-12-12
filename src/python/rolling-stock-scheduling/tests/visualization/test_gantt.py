from rolling_stock_scheduling.data.access import PkgDataAccess
from rolling_stock_scheduling.io.reader import import_response
from rolling_stock_scheduling.visualization.gantt import respone_to_gantt


def test_response_to_gant():
    response = import_response(PkgDataAccess.locate_response())
    respone_to_gantt(response, "Rolling stock schedule")
