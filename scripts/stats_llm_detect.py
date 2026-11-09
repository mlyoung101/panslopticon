# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
import pandas as pd
import psycopg
from nltk.corpus import stopwords
import nltk
import string
from scipy.stats import ttest_ind
from rich import print
import swifter
from transformers import GPT2LMHeadModel, GPT2TokenizerFast
from accelerate import Accelerator
import torch


# these two from
# https://github.com/shivamkapoor172002/Language-Model-Text-Analysis-with-Burstiness-and-Perplexity/blob/main/app.py
def preprocess_text(text):
    tokens = nltk.word_tokenize(text.lower())
    stop_words = set(stopwords.words("english"))
    tokens = [
        token
        for token in tokens
        if token not in stop_words and token not in string.punctuation
    ]
    return tokens


def calculate_burstiness(text):
    tokens = preprocess_text(text)
    word_freq = nltk.FreqDist(tokens)

    avg_freq = sum(word_freq.values()) / len(word_freq)
    variance = sum((freq - avg_freq) ** 2 for freq in word_freq.values()) / len(
        word_freq
    )

    burstiness_score = variance / (avg_freq**2)
    return burstiness_score

# from: https://huggingface.co/docs/transformers/perplexity
device = Accelerator().device
model_id = "openai-community/gpt2-large"
model = GPT2LMHeadModel.from_pretrained(model_id).to(device)
tokenizer = GPT2TokenizerFast.from_pretrained(model_id)

def compute_perplexity(text):
    encodings = tokenizer(text, return_tensors="pt")
    max_length = model.config.n_positions
    stride = 512
    seq_len = encodings.input_ids.size(1)

    nll_sum = 0.0
    n_tokens = 0
    prev_end_loc = 0
    for begin_loc in range(0, seq_len, stride):
        end_loc = min(begin_loc + max_length, seq_len)
        trg_len = end_loc - prev_end_loc  # may be different from stride on last loop
        input_ids = encodings.input_ids[:, begin_loc:end_loc].to(device)
        target_ids = input_ids.clone()
        target_ids[:, :-trg_len] = -100

        with torch.no_grad():
            outputs = model(input_ids, labels=target_ids)

            # loss is calculated using CrossEntropyLoss which averages over valid labels
            # N.B. the model only calculates loss over trg_len - 1 labels, because it internally shifts the labels
            # to the left by 1.
            neg_log_likelihood = outputs.loss

        # Accumulate the total negative log-likelihood and the total number of tokens
        num_valid_tokens = (target_ids != -100).sum().item()  # number of valid tokens in target_ids
        batch_size = target_ids.size(0)
        num_loss_tokens = num_valid_tokens - batch_size  # subtract batch_size due to internal label shift
        nll_sum += neg_log_likelihood * num_loss_tokens
        n_tokens += num_loss_tokens

        prev_end_loc = end_loc
        if end_loc == seq_len:
            break

    avg_nll = nll_sum / n_tokens  # average negative log-likelihood per token
    ppl = torch.exp(avg_nll)
    return float(ppl)

if __name__ == "__main__":
    with psycopg.connect("host=lagoon user=postgres dbname=panslopticon") as conn:
        print("Collecting data...")
        slop = pd.read_sql_query(
            """
            SELECT * FROM full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT 50
            """,
            conn,
        )
        ham = pd.read_sql_query(
            """
            SELECT * FROM ham_full_text WHERE LOWER(file) = 'readme.md' ORDER BY RANDOM() LIMIT 50
            """,
            conn,
        )

        print("[green]Compute burstiness scores...[/green]")
        slop["burstiness"] = slop.swifter.apply(
            lambda row: calculate_burstiness(row["text"]), axis=1
        )
        ham["burstiness"] = ham.swifter.apply(
            lambda row: calculate_burstiness(row["text"]), axis=1
        )

        print("[green]Compute perplexity scores...[/green]")
        slop["perplexity"] = slop.swifter.apply(
            lambda row: compute_perplexity(row["text"]), axis=1
        )
        ham["perplexity"] = ham.swifter.apply(
            lambda row: compute_perplexity(row["text"]), axis=1
        )

        print("Slop:")
        print(slop)
        print("Ham:")
        print(ham)

        result = ttest_ind(
            a=slop["burstiness"], b=ham["burstiness"], equal_var=False
        ).pvalue
        print(f"burstiness p < {result:.16f}")

        print(f"Slop burstiness mean: {slop['burstiness'].mean()}")
        print(f"Ham burstiness mean: {ham['burstiness'].mean()}")
