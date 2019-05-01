---
layout: post
title:  "Shader games with Vulkan"
description: What in tarnation?
date:   2019-01-10 05:00:00 -0500
categories: python vulkan
excerpt_separator: <!--more-->
---

Hey there. Wanna build games out of shaders? They have been around for some time now. For example, there's quite the list [on this website](https://shadertoyunofficial.wordpress.com/2017/11/11/playable-games-in-shadertoy). Of course, the scope was always very limited because the technology was not meant to do this. To be fair, it is still the case today, but you'll see that it's no longer 100% true with Vulkan.

<!--more-->

## About

The goal of this project is to move as much logic as possible from the CPU to the GPU. As a demo, I've decided to create an Asteroids clone ([wiki link](https://en.wikipedia.org/wiki/Asteroids_(video_game))) as it includes two of the features I wanted to see the most on the GPU: **draw call management** and **assets management** without adding too much clutter.

I'll spoil the beans right away, the magic is done my exposing the vertex buffer, the indices buffer, and the indirect draw buffer in the compute shaders. Once this is done, it's just a matter of recording a simple command buffer **once** and submitting it as many time as required. Continue reading to learn how it was done.

Before going into the details, the source code is available at <https://github.com/gabdube/asteroids-shader> . This article will also pinpoint the interesting parts.

## A high level overview

The project contains 4 files:

* [`asteroids.py`](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py) - The intial vulkan setup
* [`asteroids_init.comp`](https://github.com/gabdube/asteroids-shader/blob/master/asteroids_init.comp) - A compute shader that initialize the game data
* [`asteroids.comp`](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp) - A compute shader that execute a frame in the game
* [`asteroids.vert`](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.vert) / [`asteroids.frag`](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.frag) - Pass through vertex and fragment shaders

In the setup script, because all the fun stuff is done on the GPU, all that's left is a simple [check list](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1911).

After every Vulkan resource is ready to be used, it's time to record the game command buffer. This is done once per framebuffer in [the record function](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1548). The commands are recorded in the following order:

1. Bind the compute pipeline
2. Execute the game compute shader
3. Use a barrier to make sure the execution is done before rendering
4. Begin the render pass
5. Bind all the required resources
6. Execute the available `CmdDrawIndexedIndirectCount*` function

**A important thing to remember** is that `CmdDrawIndexedIndirectCount*` is a function that's pretty recent (on non-AMD hardware). It depends on the extension `VK_KHR_draw_indirect_count` [link](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VK_KHR_draw_indirect_count.html) which, according to GPUINFO, is only available in very recent graphics driver [link](https://vulkan.gpuinfo.org/listdevices.php?extension=VK_KHR_draw_indirect_count).

Before the [run loop](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1738) begins, the initialization compute shader is recorded and executed once [here](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1620).

Once the initalization is done, the script listen to the system events and calls the [render function](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1688) as fast as possible. Just before the game command buffer is executed, the game state is updated with the [update function](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1652).

There you go. That's the meat of the runtime.

## Vulkan initialization

The python setup script initialize the all vulkan resources. At 2000 lines of code, it might seems like alot, but that's just how Vulkan is.
