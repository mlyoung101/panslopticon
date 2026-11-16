# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle
import sqlite3

import matplotlib.pyplot as plt
from IPython import embed
from sentence_transformers import SentenceTransformer
from sklearn.decomposition import PCA

LIMIT = 1000


def load_data() -> tuple[list, list]:
    conn = sqlite3.connect("data/panslop.db")
    spam = conn.execute(
        f"SELECT text FROM full_text WHERE lower(file) = 'readme.md' ORDER BY RANDOM() LIMIT {LIMIT}"
    ).fetchall()
    ham = conn.execute(
        f"SELECT text FROM ham_full_text WHERE lower(file) = 'readme.md' ORDER BY RANDOM() LIMIT {LIMIT}"
    ).fetchall()
    conn.close()

    return [x[0] for x in spam], [x[0] for x in ham]


def cluster():
    print("Loading data...")
    spam, ham = load_data()

    transformer = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")
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
    ax.scatter(spam_pca[:, 0], spam_pca[:, 1], s=20, label="Slop", alpha=0.5)
    ax.scatter(ham_pca[:, 0], ham_pca[:, 1], s=20, label="Ham", alpha=0.5)
    ax.set_title("Slop. vs ham sentence embedding PCA")
    # https://stackoverflow.com/a/53219684/5007892
    # fig.subplots_adjust(left=0, bottom=0, right=1, top=1, wspace=0, hspace=0)
    # plt.tight_layout(pad=-5)
    ax.legend()
    plt.savefig("plot.pdf", bbox_inches="tight")
    plt.show()

    embed()


if __name__ == "__main__":
    cluster()
