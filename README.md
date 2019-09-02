# URLSplit

`urlsplit` is a command line tool for splitting up URLs into component pieces.

Its kind of like having python's `urllib.parse.urlparse` or rust's `url::Url::parse`
but in command line form. It takes in URLs on individual lines (possibly from a CSV)
and spits out all the parts in CSV format.

It works great when combined with [`xsv`](https://github.com/BurntSushi/xsv), a great 
library for handling large CSVs on the command line.