# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import sqlite3
import matplotlib.pyplot as plt

import numpy as np
import pandas as pd
import rich
from IPython import embed
import seaborn as sns


def main() -> pd.DataFrame:
    print("Loading data...")
    conn = sqlite3.connect("data/panslop.db")
    spam = pd.read_sql_query("SELECT * FROM slop", conn)
    ham = pd.read_sql_query("SELECT * FROM ham", conn)
    conn.close()

    sns.set(style="darkgrid")
    spam["origin_src"] = spam["origin_src"].str.replace("tag-", "")
    ham["origin_src"] = ham["origin_src"].str.replace("tag-", "")

    # fucking insane
    # https://stackoverflow.com/a/46624802/5007892
    sns.countplot(
        ham, x="origin_src", order=ham["origin_src"].value_counts().index
    ).set(title="Ham")
    # sns.countplot(
    #     spam, x="origin_src", order=spam["origin_src"].value_counts().index
    # ).set(title="spam")
    plt.xticks(rotation=90)
    plt.show()


if __name__ == "__main__":
    main()
