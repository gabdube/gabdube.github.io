---
layout: post
title:  "Python and Vulkan: The good, the bad, and what's next"
description: Builing a framework embryo with python and vulkan
date:   2019-01-10 05:00:00 -0500
categories: python vulkan
excerpt_separator: <!--more-->
---

Hey there. So you are interested in building a vulkan application in something else than c++? Why not python?
In this article, you will learn about my experiences about building a simple rendering application with both of these guys.

<!--more-->

## About the application

Codename "PanicPanda", source code available here: [panic-panda](https://github.com/gabdube/panic-panda), is a small framework embryo
that I've created to quickly tests ways to create 3D applications using Vulkan. In its current state, it can: load basic
assets (shaders, models, textures), render objects with a simple forward rendering pipeline, and it provide a sane way to manage
shaders uniform from the user side.

The project itself is 100% written in python (ie: no C extensions) and do not require any external modules for the runtime ( that was one of my
requirements). When it comes to asset management, the project depends on a variety of non-python open source projects. All which have been wrapped
in python helpers for convenience.

The end goal of the project is to create a simple 3D game with sounds, physics and animations included. At some point, the no c extensions will be
lifted, because I can't see myself implementing a sound system directly in python.

![Octocat](/assets/images/demo.png)

## The good

### Fast debugging cycle

In case you didn't know, Vulkan is kind of complicated. There's a lot of things that can possibly go wrong. In this scenario, the run-compile-test loop becomes pretty annoying fast. Python being interpreted, my build time is still under 1 sec. The biggest hurdle is not even related to the language, it's
uploading high quality images to the device memory.

Also with python, it's very easy to edit values at runtime. It's just a matter of evaluating new statements.

At some point I'd love to directly code into my application (ie: without having to close the app and reupload the resources). Reloading loaded modules
sounds easy in theory, but I'm not sure if it will be worth it.

### It's easier to manage vulkan resources

With python being garbage collected, it's easier to focus on managing the vulkan resources. Vulkan has many different type of resources and many different way to allocate them, so when toying with Vulkan, anything that can lessen this burden should be welcomed.

Python alone is still not enough though, it must be coupled with the instance validation layers to make sure no resources remains unfreed. Cause you never know when
a descriptor set or a fence may slipt between your fingers.

### Writting a sane user interface is simple

Python being super high level, it possible redefine some basic function such as writing and reading fields. This is particuliary great when I designed
my uniform system. I've managed to hide the complexity of the Vulkan uniforms system away from the end user, and believe me, that was hard. Uniforms are 
definitly one of the most complicated topics of the Vulkan API.

Duck typing is also pretty usefull when you are not exactly sure where everything will go in the end. For now at least, it has made refractoring a lot easier for me.

## The bad

### There's no light tooling available

The current ecosystem built around Vulkan is pretty much exclusively C++. With maybe some experimental wrappers for other languages here and there.
Be ready to write lots of boilerplate code, and with Vulkan not being shy in that regard in the first place, things can get painful. Hopefully for me, 
I've maintained my own little wrapper with its set of helpers.

Another thing is that python lacks light libraries in many regards: in linear algebra, in image loading, and in assets loading. Want to multiply two matrices, here's scipy and numpy. Now cry and despair as you try to build the C dependencies on Windows. In the end, I've ended up writing my own math library and assets loaders. It wasn't as hard as I though it would be.

And finally, although not really related to Vulkan and python, assets management is pretty difficult outside of a fully fledged engine environment. It
took me quite some time to build adequate tooling to compile textures and environment maps.

### The weight of the wrapper layer

PanicPanda use my own little wrapper which was built around ctypes. ctypes is the foreign function library that comes bundled in the python standard
library. While the library is quick and easy to use, it is also the slowest way to wrap c function calls. Using the c structures provided by ctypes tends to generate lots of garbage too. This will most likely become a bottleneck in the future.

There other more performing alternative like cython or just plain old C extensions that could be used, but the current state of the project, the
added complexity is not worth it.

### The scary monster under your bed, the GC

One of the biggest features of Vulkan is efficient multithreading. This is mostly done by recording the command buffers on multiple thread.
With python and the GIL, the advanges goes away. Moving the command recording outside of python, in a C extension, could solve the problem, but then
again, the added complexity is not worth it for now.

The GC might also introduce stutters later on. Maybe the high level of control that Vulkan provides will alleviate some of the problem. For example,
maybe calling the GC while the scene are being rendered will help?

## What's next

### Compute

Vulkan has a compute API.

* Can we easily use it over other existing solutions?
* Just what are the limits compared to compute-only API (opencl, CUDA)

### Rendering pipeline management

Forward rendering is boring. Just how hard can it be to implement a more modern rendering pipeline. I'm looking
at you my dear forward clustered rendering...

How can I expose a way to customize a renderpass from the end user perspective (and keep things relatively simple).

### Command buffers submitting improvements

It's in our interest to keep command buffer recording to a minimum. Just how far can reduce the driver overhead with Vulkan?

### Animations and Physics

Animations and physics are probably to two most cpu hungry features in a game engine. It woudn't make any sense to implement them
in pure python. Can we move the bulk of the computation on the GPU? Will it make a big difference?

### Performances profiling

At some point profiling performances will actually become usefull to meter.

* How much ctypes really slow down the application?
* Is PyPy a viable alternative?
* Lets rewrite the app in Rust, what will the performances improvement look like?

## Ending note

Was it worth it? Yes.

Is the project done? No.

Will writing heavy 3D applications in pure python become viable? Probably not. But it's too soon to be sure.

As of this date, will I be writing my next science-based dragon MMO in python? No. My eyes are set on Rust for this.
