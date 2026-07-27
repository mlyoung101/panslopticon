# Copyright (c) 2026 Mel Young.
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL
# was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.
using TextAnalysis: serialize, deserialize
using SQLite
using DataFrames
using WordTokenizers
using StatsBase
using Serialization
using MLDataUtils
using TextAnalysis
using ProgressMeter

# Evaluates the classifer after running classify_train.jl

function evaluate()
    println("Loading data...")
    test_ham = deserialize("data/test_ham.dat")
    test_spam = deserialize("data/test_spam.dat")
    classifier::NaiveBayesClassifier = deserialize("data/classifier.dat")

    for ham in test_ham
        println(TextAnalysis.predict(classifier, ham)[:ham])
    end

    println("Done.")
end

evaluate()
