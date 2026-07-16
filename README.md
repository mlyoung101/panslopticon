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
### Detection methodology
For obvious AI usage: (At least 1 AI tool commit OR At least 1 AI file) AND (Matches at least 1 README signal)

For less obvious usage: Has excessive commits AND Matches at least 3 README signals

### Tasks
**IngressGitHub:** Runs once every 3 hours. Scrapes GitHub trending page for repositories made since January 1
2024 for investigation.

**IngressReddit:** Runs once per day at 7am AEDT. Scrapes various subreddits by new to locate slop.

**Analyse:** Runs once every 6 hours. Visits the ingress queue and determines if repositories are actually
slop or not. If yes, they get added to the slop table and catalogued. If no, they get added to the "not_slop"
table and will not be visited again.

**UpdateStats:** Runs once per day at 11pm AEDT. Looks through all existing slop repositories to check if they
still exist and updates their stars and forks count.

## Licence
Copyright (c) 2026 Mel Young. Available under the MPL 2.0.

As might be expected, no AI was used in the making of this project...
