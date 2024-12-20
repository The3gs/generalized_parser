# Complex parser with support for arbitrary operators

This is a simple (relatively speaking) implementation of a pratt parser with support for arbitrary mixfix operators.
I made this for my learning and experience, so it isn't the most idomatic yet.

## Resources

These are some resources I found helpful in understanding and implementing this:
* [Parsing Mixfix Operators](https://www.cse.chalmers.se/~nad/publications/danielsson-norell-mixfix.pdf) by Nils Danielson and Ulf Norell
* [Simple but powerful Pratt Parser](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) by Alex Kladov

## Whats next

I would like to update the parser to use a partial relation as in the Parsing Mixfix Operators Paper, as I think that works better for composibility with libraries.

I think it is possible to make the parser add identifiers as it parses, which would allow for parsing expressions likes Coq's `exists x , x + x = 2` into `(exists_,_ (lambda x (_=_ (_+_ (x) (x)) (2))))`
Unfortunately that does not allow the grouping operation to be a simpl preprocessing step, as it currently is.
