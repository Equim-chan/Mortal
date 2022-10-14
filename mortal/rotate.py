import prelude

from datetime import timedelta, datetime, date, time
import logging
import os
from time import sleep
from config import config


def cp(source, target):
    if os.path.exists(source):
        with open(source, "rb") as source_file, open(target, "wb") as tar_file:
            logging.info("{} -> {}".format(source, target))
            tar_file.write(source_file.read())


def rotate():
    curent = config["control"]["state_file"]
    for i in range(5):
        yesterday = os.path.dirname(curent) + "/T-{}.pth".format(i + 1)
        old_archive = (
            os.path.dirname(curent)
            + "/"
            + (datetime.now() + timedelta(days=(-(i + 1)))).strftime("%Y%m%d")
            + ".pth"
        )
        cp(old_archive, yesterday)

    archive = os.path.dirname(curent) + "/" + datetime.now().strftime("%Y%m%d") + ".pth"
    cp(curent, archive)


def sleep_to_dawn():
    now = datetime.today()

    next_day = date.today() + timedelta(days=1)
    dawn_time = time(hour=6, minute=30)
    next_dawn = datetime.combine(next_day, dawn_time)
    delt = next_dawn - now
    delt = delt.total_seconds()
    logging.info("sleeping {} seconds".format(delt))
    sleep(delt)


def main():
    while True:
        rotate()
        sleep_to_dawn()


if __name__ == "__main__":
    main()
