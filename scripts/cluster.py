# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle
import sqlite3
import matplotlib.pyplot as plt
from IPython import embed
from sklearn.decomposition import PCA

from sentence_transformers import SentenceTransformer

LIMIT = 2000


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


def cluster():
    print("Loading data...")
    spam, ham = load_data()

    transformer = SentenceTransformer(
        "sentence-transformers/all-MiniLM-L6-v2", cache_folder="/var/tmp/transformers"
    )
    spam_transformed = transformer.encode(spam, show_progress_bar=True)
    ham_transformed = transformer.encode(ham, show_progress_bar=True)

    with open("data/spam_transformed.dat", "wb") as f:
        pickle.dump(spam_transformed, f)

    with open("data/ham_transformed.dat", "wb") as f:
        pickle.dump(ham_transformed, f)

    # perform PCA
    print("Perform PCA")
    spam_pca = PCA(n_components=2).fit_transform(spam_transformed)
    ham_pca = PCA(n_components=2).fit_transform(ham_transformed)

    fig = plt.figure(1, figsize=(8, 6))
    ax = fig.add_subplot(111)
    ax.scatter(
        spam_pca[:, 0],
        spam_pca[:, 1],
        s=40,
        label="Spam"
    )
    ax.scatter(
        ham_pca[:, 0],
        ham_pca[:, 1],
        s=40,
        label="Ham"
    )
    ax.legend()
    plt.show()

    embed()


if __name__ == "__main__":
    cluster()
