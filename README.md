# Panslopticon
The goal of Panslopticon is to build a comprehensive dataset of "AI slop" code projects on GitHub for research
purposes.

It's well known that in our brave new world, this type of development is considered to be the future and what
we should all apparently be aspiring to do. So, given this, the Panslopticon project aims to build a
comprehensive dataset of these projects to understand what our future might look like.

This can allow us to answer questions about these projects like:
- How long are they active for?
- How many contributors do they have?
- What is the most common programming language they are written in?
- What are the most commonly used AI tools?
- Is it true that AI-written READMEs have a particular style that can easily be detected?

"AI slop" is detected through regex heuristics, and so is not perfect. You shouldn't be offended if your
project ends up in the dataset, the heuristics are not perfect. However, I do aim to have near zero false
positives at the trade-off of many false negatives.

## Setup
Requires Rust >= 1.97 and the Rust `sqlx` CLI.

Setup the database:

```
sqlx database create
sqlx migrate run
```

## Architecture
### Terminology
**Slop** refers to poor-quality, entirely AI-generated repos; which is what we are aiming to detect. It's also
referred to as "spam" in some places in the code (because we're building a classifier in the classic
"spam"/"ham" sense).

**Ham** refers to good-quality, human-authored repos of good standing. This is what we used to train the
classifier on the opposite of slop.

**Not Slop** in this codebase, counter-intuitively means content that we _think_ is not slop. If we are _sure_
it's not slop (it has a very low score and was created before the cutoff date), it would instead be ham.

### Detection methodology
Detection is configured through `config.toml`, though a number of regexes and other heuristics. The scoring
system looks at the README contents, creation date, files in the repo (including gitignored files!) and commit
authorship. All of these are considered "signals", it takes a number of signals to increase the score
significantly.

Once the score is above a configurable threshold, the repo is considered slop.

### Daily tasks
**IngressGitHub:** Runs once every 3 hours. Uses the GitHub API to query various topics for repos, sorted by
most recently updated, with greater than 2 stars.

**IngressHam:** Uses the GitHub API to query topics for repos, sorted by most stars, with >90 stars and
created before 2022; picking a random page.

**Analyse:** Runs once every day at ~10pm AEDT. Visits the ingress queue and determines if repositories are
actually slop or not. If yes, they get added to the slop table and catalogued (e.g. by having their full text
from all files archived). If no, they get added to the "not_slop" table and will not be visited again.

### Non-daily tasks
**Update:** Looks through all existing slop repositories to check if they
still exist and updates their stars and forks count.

**Reconsider:** Reconsiders content in the `not_slop` table; useful if the classification algorithm has been
updated to be more aggressive or thresholds have been changed.

## Spam/ham classifier
Some classifiers are prototyped in Python to attempt to distinguish between human-written READMEs and slop
READMEs, i.e. the classic ham vs. spam task. Currently, a simple naive Bayes classifier performs excellently
on the dataset.

## Licence
Copyright (c) 2026 Mel Young. Available under the MPL 2.0.

As might be expected, no AI was used in the making of this project...
