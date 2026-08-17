# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle
import sqlite3
import matplotlib.pyplot as plt
from IPython import embed
from sklearn.decomposition import PCA
import evaluate
from tqdm import tqdm


LIMIT = 255


def load_data() -> tuple[list, list]:
    conn = sqlite3.connect("data/panslop.db")
    spam = conn.execute(
        f"SELECT text FROM full_text ORDER BY RANDOM() LIMIT {LIMIT}"
    ).fetchall()
    ham = conn.execute(
        f"SELECT text FROM ham_full_text ORDER BY RANDOM() LIMIT {LIMIT}"
    ).fetchall()
    conn.close()

    return [x[0] for x in spam], [x[0] for x in ham]


def check():
    print("Loading data...")
    spam, ham = load_data()
    perplexity = evaluate.load("perplexity", module_type="metric", cache_dir="/var/tmp/evaluate")

    print("Spam perplexity:")
    for item in spam:
        print(item)
        results = perplexity.compute(model_id="Qwen/Qwen3-0.6B", add_start_token=False, predictions=[item])
        print(results)

    embed()



if __name__ == "__main__":
    check()
