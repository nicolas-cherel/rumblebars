# rumblebars - a handlebars template evaluation library

Rumblebars passes all mustaches specs[1] and 272 handlebars tests[2]. Template evaluation is rendered to a io::Writer, so that you can choose wether if you hold result in memory or not. It also input data angostic, given that your data structure implements the HBData trait (implementation for Json provided).

[1] minus one test due to a trailing space
[2] all tests that does not involves javascript in data and partials, and see the comments for other cases here : https://github.com/nicolas-cherel/rumblebars/blob/master/tests/eval/handlebars.rs#L88-L134
