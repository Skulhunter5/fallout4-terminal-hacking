#!/usr/bin/env python
from wordfreq import top_n_list

def main(out_dir):
    wordlists = generate_wordlists([4, 5])
    save_wordlists(out_dir, wordlists)

def generate_wordlists(letter_counts):
    words = top_n_list('en', 10000)
    wordlists = {}
    for letter_count in letter_counts:
        wordlists[letter_count] = []
    for word in words:
        letter_count = len(word)
        if letter_count in letter_counts:
            wordlists[letter_count].append(word)
    return wordlists

def save_wordlists(out_dir, wordlists):
    for letter_count, wordlist in wordlists.items():
        save_wordlist(out_dir, letter_count, wordlist)

def save_wordlist(out_dir, letter_count, wordlist):
    path = out_dir + "/words" + str(letter_count) + ".txt"
    with open(path, "w") as file:
        for word in wordlist:
            file.write(word + "\n")

if __name__ == '__main__':
    import sys
    if len(sys.argv) != 2:
        print("incorrect usage")
        exit(1)
    out_dir = sys.argv[1]

    main(out_dir)
