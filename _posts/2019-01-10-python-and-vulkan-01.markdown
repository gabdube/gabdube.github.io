---
layout: post
title:  "Python and Vulkan: The good, the bad, and what's next"
description: Builing a framework embryo with python and vulkan
date:   2019-01-10 05:00:00 -0500
categories: python vulkan
excerpt_separator: <!--more-->
---

Hey there. So you are interested in building a vulkan application in something else than c++? Why not python?
In this article, you will learn about my experiences about building a simple rendering application with these tools.

<!--more-->

## About the application

Codename "PanicPanda", source code available here: [panic-panda](https://github.com/gabdube/panic-panda), is a small framework embryo
that I've created to quickly tests ways to create 3D applications using Vulkan. In its current state, it can: load basic
assets (shaders, models, textures), render objects with a simple forward rendering pipeline, and it provide a sane way to manage
shaders uniform from the user side.

The project itself is 100% written in python (ie: no C extensions) and do not require any external modules for the runtime ( that was a requirement I've given to myself ). When it comes to asset management, the project depends on a variety of non-python open source projects. All which have been wrapped in python helpers for convenience.

The end goal of the project is to create a simple 3D game with sounds, physics and animations included. At some point, the no c extensions will be
lifted, because I can't see myself implementing a sound system directly in python.

![Octocat](/assets/images/demo.png)

## The good

### Fast debugging cycle

### Writting a sane user interface is simple

### Running on both Windows and Linux has never been easier

### Worrying about CPU performances, why not put in on the GPU instead?

## The bad

### There's no light tooling available

The current ecosystem built around Vulkan is pretty much exclusively C++. With maybe some experimental wrappers for other languages here and there.
Be ready to write lots of boilerplate code, and with Vulkan not being shy in that regard in the first place, things can get painful. Hopefully for me, 
I've maitained my own little wrapper with its set of helpers.

Another thing is that python do not have a light library to do linear algebra, or light image loading or light assets loading. Want to multiply two matrices, here's scipy and numpy. Now cry and despair as you try to build the C dependencies on Windows. I ended up writing my own math library and
loaders.

And finally, although not really related to Vulkan and python, assets management is pretty difficult outside of an engine environment. It took me quite
some time to build adequate tooling to compile textures and environment map.

### The weight of the wrapper layer

PanicPanda use my own little wrapper which was built around ctypes. ctypes is the foreign function library that comes bundled in the standard library.
While the library is quick and easy to use, it is also the slowest way to wrap c function calls. Using the c structures provided by ctypes tends to generate lots of garbage too. This will probably become a bottleneck in the future.

There other more performing alternative like cython or just plain old C extensions that could be used, but the current state of the project, the
added complexity is not worth it.

### The scary monster under your bed, the GC

One of the biggest features of Vulkan is efficient multithreading. This is mostly done by recording the command buffers on multiple thread.
With python and the GIL, the advanges goes away. Moving the command recording outside of python, in a C extension, could solve the problem, but then
again, the added complexity is not worth it for now.

## What's next

### Compute

Vulkan has a compute API.

* Can we easily use it over other existing solutions?
* Just what are the limits compared to compute-only API (opencl, CUDA)

### Rendering pipeline management

Forward rendering is boring. Just how hard can it be to implement a more modern redering pipeline. I'm looking
at you my dear forward clustered rendering...

### Command buffers submitting improvements

It's in our interest to keep command buffer recording to a minimum. Just how far can reduce the driver overhead with Vulkan?

### Animations and Physics

Animations and physics are probably to two most cpu hungry features in a game engine. It woudn't make any sense to implement them
in pure python. Can we move the bulk of the computation on the GPU.

### Performances profiling

At some point profiling performances will actually become usefull to meter.

* How much ctypes really slow down the application?
* Is PyPy a viable alternative?
* Lets rewrite the app in Rust, what will the performances improvement look like?
