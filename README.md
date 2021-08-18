# Krustlet WAGI Provider

**WARNING:** This is experimental code.
It is not considered production-grade by its developers, neither is it "supported" software.

This is a [Krustlet](https://github.com/deislabs/krustlet) [Provider](https://github.com/deislabs/krustlet/blob/master/docs/topics/architecture.md#providers) implementation for the [WAGI](https://github.com/deislabs/wagi) runtime.

## Documentation

If you're new to Krustlet, get started with [the
introduction](https://github.com/deislabs/krustlet/blob/master/docs/intro/README.md) documentation.
For more in-depth information about Krustlet, plunge right into the [topic
guides](https://github.com/deislabs/krustlet/blob/master/docs/topics/README.md).

# Running the Demo

In a new terminal, start the WAGI provider:
```
$ just run
```

In another terminal:
```
$ kubectl apply -f demos/wagi/k8s.yaml
```

Once the pod is ready:
```
$ curl 127.0.0.1:3000/hello
hello world
```