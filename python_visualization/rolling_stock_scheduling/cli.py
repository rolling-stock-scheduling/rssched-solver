from pathlib import Path
from typing import Annotated

import typer
from typer import Argument, echo

from rolling_stock_scheduling.io.reader import import_response
from rolling_stock_scheduling.visualization.gantt import respone_to_gantt

app = typer.Typer()


@app.command()
def main(source: Annotated[Path, Argument()]):
    echo(f"Render visualization: {source}")
    response = import_response(source)
    respone_to_gantt(response, f"Rolling stock schedule: {source.stem}").show()


if __name__ == "__main__":
    app()
