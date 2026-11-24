# GENERATED!!!
import pickle
import sys

import matplotlib
import rich
from rich.text import Text
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.naive_bayes import MultinomialNB

BAR_WIDTH = 50


def turbo_hex(t: float) -> str:
    rgba = matplotlib.colormaps["turbo"](t)
    return "#{:02x}{:02x}{:02x}".format(
        int(rgba[0] * 255), int(rgba[1] * 255), int(rgba[2] * 255)
    )


def print_gradient(slop_prob: float):
    t = slop_prob / 100.0
    bar = Text("ham ", style="green")
    for i in range(BAR_WIDTH):
        bar.append(" ", style=f"on {turbo_hex(i / (BAR_WIDTH - 1))}")
    bar.append(" slop", style="red")
    marker_pos = round(t * (BAR_WIDTH - 1))
    marker = Text("    " + " " * marker_pos + "^", style=f"bold {turbo_hex(t)}")
    rich.print(bar)
    rich.print(marker)


def classify(word: str):
    with open("data/tfidf_vectorizer.dat", "rb") as f:
        vectorizer: TfidfVectorizer = pickle.load(f)

    with open("data/classifier_MultinomialNB.dat", "rb") as f:
        classifier: MultinomialNB = pickle.load(f)

    classes = list(classifier.classes_)
    spam_idx = classes.index("spam")
    ham_idx = classes.index("ham")

    analyze = vectorizer.build_analyzer()
    tokens = analyze(word)
    known = [tok for tok in tokens if tok in vectorizer.vocabulary_]
    if not known:
        rich.print(
            f"[yellow]'{word}' is not in the classifier's vocabulary "
            f"(out-of-vocabulary); cannot classify.[/yellow]"
        )
        return

    features = vectorizer.transform([word])
    prediction = classifier.predict_proba(features)
    ham_prob = prediction[0][ham_idx] * 100.0
    slop_prob = prediction[0][spam_idx] * 100.0

    t = slop_prob / 100.0
    color = turbo_hex(t)
    verdict = "ham" if ham_prob >= slop_prob else "slop"
    chance = max(ham_prob, slop_prob)

    rich.print(
        f"[bold {color}]'{word}' is {verdict} ({chance:.2f}% chance)[/bold {color}]"
    )
    print_gradient(slop_prob)
    rich.print(f"[green]{ham_prob:.2f}% ham[/green] / [red]{slop_prob:.2f}% slop[/red]")


def interactive():
    rich.print(
        "[bold]TF-IDF Naive Bayes slop/ham query[/bold] (empty line or Ctrl-D to quit)"
    )
    while True:
        try:
            word = input("Word: ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break
        if not word:
            break
        classify(word)


if __name__ == "__main__":
    print("Hi")
    if len(sys.argv) >= 2:
        classify(" ".join(sys.argv[1:]))
    else:
        interactive()
