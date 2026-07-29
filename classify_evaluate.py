# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pickle

from IPython import embed
import rich
from sklearn.ensemble import AdaBoostClassifier, RandomForestClassifier
from sklearn.feature_extraction.text import CountVectorizer, TfidfVectorizer
from sklearn.linear_model import LogisticRegressionCV, LinearRegression
from sklearn.metrics import accuracy_score, classification_report, log_loss
from sklearn.model_selection import train_test_split
from sklearn.naive_bayes import MultinomialNB
from sklearn.neighbors import KNeighborsClassifier
from sklearn.svm import SVC
from sklearn.tree import DecisionTreeClassifier
from sklearn import tree
from sklearn.decomposition import PCA
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
    rich.print(f"[bold]Prediction:[/bold]\n[green]{ham_prob:.4f}% ham[/green]\n[red]{spam_prob:.4f}% spam[/red]")

    # for name, classifier in classifiers.items():
    #     print(f"Fitting classifier {name}...")
    #     classifier.fit(X_train_vect, y_train)
    #
    #     print("Predicting...")
    #     y_pred = classifier.predict(X_test_vect)
    #     report = classification_report(y_test, y_pred)
    #
    #     rich.print(f"[bold red]Classification report for {name}:[/bold red]\n{report}")
    #     print()
    #
    #     with open(f"data/classifier_{type(classifier).__name__}.dat", "wb") as f:
    #         pickle.dump(classifier, f)
    #
    #     if isinstance(classifier, DecisionTreeClassifier):
    #         with open("/tmp/tree.dot", "w") as f:
    #             tree.export_graphviz(
    #                 classifier,
    #                 out_file=f,
    #                 feature_names=vectorizer.get_feature_names_out(),
    #                 class_names=classifier.classes_,
    #                 filled=True,
    #             )
    #
    #     if isinstance(classifier, MultinomialNB):
    #         important_features(vectorizer, classifier, 20)


if __name__ == "__main__":
    load(sys.argv[1])
