# ssb-flumeview-sql 

> Node bindings to to sql flumeview on a ssb database 

- differs from the current idea of a flumeview
  - doesn't plug into a js flume log, uses it's own internal rust based flumelog
  - read only, points at the offset file (log.offset)
  - supports building the view in chunks to control cpu time and allow queries to be handled in between building the view.

- supports querying the view, even if the view is behind the log.


## API

```js
```


## Install

With [npm](https://npmjs.org/) installed, run

```
$ npm install 
```

## Building

### Build for your environment

Dev Dependencies:
  - rust + cargo
  - cargo-make
  - cmake

```
$ npm run prebuild
```

### Cross compile

Dev Dependencies
  - same as above
  - cross
  - dockcross

Cross compiling is still a work in progress. Still todo:
  - [ ] split out scripts to cross in build-native.sh
  - [ ] add script to tar up cross compiled binding with correct prebuild compatible filename
  - [ ] windows builds don't work

## Acknowledgments

## See Also

## License

AGPL3
