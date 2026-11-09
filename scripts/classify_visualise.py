# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import html
import pickle
import re
import sys

import matplotlib
import matplotlib.colors as mcolors
import numpy as np
import requests
import rich
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.naive_bayes import MultinomialNB


def visualise(url: str, output_path: str = "report.html"):
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

    classes = list(classifier.classes_)
    spam_idx = classes.index("spam")
    ham_idx = classes.index("ham")

    features = vectorizer.transform([content])
    prediction = classifier.predict_proba(features)
    ham_prob = prediction[0][ham_idx] * 100.0
    spam_prob = prediction[0][spam_idx] * 100.0
    rich.print(
        f"[bold]Prediction:[/bold]\n[green]{ham_prob:.4f}% ham[/green]\n[red]{spam_prob:.4f}% spam[/red]"
    )

    # Compute posterior slop probabilities for vocabulary words
    log_prob = classifier.feature_log_prob_ + classifier.class_log_prior_[:, None]
    prob = np.exp(log_prob - np.max(log_prob, axis=0, keepdims=True))
    prob = prob / np.sum(prob, axis=0, keepdims=True)
    word_slop = {
        word: prob[spam_idx, idx] for word, idx in vectorizer.vocabulary_.items()
    }

    cmap = matplotlib.colormaps["jet"]

    parts = []
    for token in re.split(r"(\b\w+\b)", content):
        p = word_slop.get(token.lower())
        if p is not None and p > 0.5:
            intensity = (p - 0.5) / 0.5
            rgba = cmap(intensity)
            hex_color = mcolors.to_hex(rgba)
            text_color = (
                "white"
                if (rgba[0] * 0.299 + rgba[1] * 0.587 + rgba[2] * 0.114) < 0.5
                else "black"
            )
            escaped = html.escape(token)
            parts.append(
                f'<span style="background-color: {hex_color}; color: {text_color}; border-radius: 3px; padding: 1px 2px;" '
                f'title="{p * 100:.1f}% slop">{escaped}</span>'
            )
        else:
            parts.append(html.escape(token))

    rendered_text = "".join(parts)
    legend_stops = ", ".join(mcolors.to_hex(cmap(i / 10)) for i in range(11))

    html_report = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Slop Classification Report - {html.escape(url)}</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            margin: 2rem auto;
            max-width: 900px;
            padding: 0 1rem;
            color: #222;
            background: #f8f9fa;
        }}
        .header {{
            background: #fff;
            padding: 1.5rem;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            margin-bottom: 1.5rem;
        }}
        .prediction {{
            display: flex;
            gap: 1.5rem;
            margin-top: 0.5rem;
            font-size: 1.1rem;
        }}
        .ham {{ color: #2e7d32; font-weight: bold; }}
        .slop {{ color: #c62828; font-weight: bold; }}
        .legend-container {{
            margin-top: 1rem;
        }}
        .legend-bar {{
            height: 16px;
            border-radius: 4px;
            background: linear-gradient(to right, {legend_stops});
            margin-bottom: 4px;
        }}
        .legend-labels {{
            display: flex;
            justify-content: space-between;
            font-size: 0.85rem;
            color: #666;
        }}
        .content {{
            white-space: pre-wrap;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 14px;
            line-height: 1.6;
            background: #fff;
            padding: 1.5rem;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            word-break: break-word;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h2>Classification Report</h2>
        <div><strong>Source:</strong> {html.escape(url)}</div>
        <div class="prediction">
            <span class="ham">Ham: {ham_prob:.2f}%</span>
            <span class="slop">Slop: {spam_prob:.2f}%</span>
        </div>
        <div class="legend-container">
            <div class="legend-bar"></div>
            <div class="legend-labels">
                <span>&le; 50% Slop (uncolored)</span>
                <span>100% Slop</span>
            </div>
        </div>
    </div>
    <div class="content">{rendered_text}</div>
</body>
</html>
"""

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html_report)

    rich.print(f"[bold]HTML report written to:[/bold] {output_path}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python scripts/classify_visualise.py <url_or_file> [output_html]")
        sys.exit(1)
    target = sys.argv[1]
    output = sys.argv[2] if len(sys.argv) > 2 else "report.html"
    visualise(target, output)
