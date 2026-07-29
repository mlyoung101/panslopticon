# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle

from IPython import embed
import rich
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.naive_bayes import MultinomialNB
import sys
import requests


def load(url: str):
    print("Loading models...")
    with open("data/tfidf_vectorizer.dat", "rb") as f:
        vectorizer: TfidfVectorizer = pickle.load(f)

    with open("data/classifier_MultinomialNB.dat", "rb") as f:
        classifier: MultinomialNB = pickle.load(f)

    # goofy
    if url.startswith("http"):
        content = requests.get(url).content.decode("utf-8")
    else:
        with open(url) as f:
            content = f.read()

    features = vectorizer.transform([content])
    prediction = classifier.predict_proba(features)
    ham_prob = prediction[0][0] * 100.0
    spam_prob = prediction[0][1] * 100.0
    rich.print(
        f"[bold]Prediction:[/bold]\n[green]{ham_prob:.4f}% ham[/green]\n[red]{
            spam_prob:.4f}% spam[/red]"
    )


if __name__ == "__main__":
    load(sys.argv[1])
