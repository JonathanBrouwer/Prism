<div align="center">
  <h1><code>Prism Parser</code></h1>
  <p><strong>A PEG parsing library build for the Prism programming language</strong></p>
</div>

## About

This crate provides a PEG parser with the following features:
* Support for left recursion
* Adaptation: The grammar can be changed dynamically during parsing

This crate is under heavy development and not yet ready to be used in production code.


## Example
This defines the grammar of a simple arithmetic language, and support to adapt the language: 
```
rule start = block;
rule block {
    b <- "grammar" "{" g:grammar(prule_action) "}" ";" b:#adapt(g, block);
    s :: b <- s:stmt ";" b:block;
    [] <- "";
}

rule stmt {
    Let(e) <- "let" e:expr;
    Do() <- "do";
}

rule expr {
    group additive {
        Add(x, y) <- x:#next "+" y:#this;
    }
    group multiplicative {
        Mul(x, y) <- x:#next "*" y:#this;
    }
    group base {
        Block(b) <- "(" b:block ")";
        UnaryMinus(v) <- "-" v:#this;
        Num(n) <- n:#str([0-9]*);
    }
}

rule layout = [' ' | '\n'];
```

Example programs in this language:
```
1 * 2 + -3
```
```
grammar {
    rule expr {
        group additive {
            1 + (-2) <- x:#next "-" y:#this;
        }
    }
};
1 - 2
```