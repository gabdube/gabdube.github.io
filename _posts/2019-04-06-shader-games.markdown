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

The python setup script initialize the all vulkan resources. At 2000 lines of code, it might seems like alot, but that's just how verbose Vulkan is.

There's nothing interesting in the instance setup.

For [the device setup](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L330), a feature and an extension must be enabled.

### Extension: Draw indirect count

Vulkan alone supports indirect drawing through the function `vkCmdDrawIndexedIndirect`. There is one problem though: there's no way to tell the GPU how many draw parameters there is in the buffer. This limitation can be removed with any of those two extensions: `VK_AMD_draw_indirect_count` and `VK_KHR_draw_indirect_count`. Both extensions work in the exact same way by exposing the `vkCmdDrawIndexedIndirectCount*` ([link](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/vkCmdDrawIndexedIndirectCountKHR.html)) function.

The difference is that `VK_AMD_draw_indirect_count` is exclusive to AMD hardware and is available since the beginning of time while `VK_KHR_draw_indirect_count` is available on any hardware vendor (including AMD), but only in recent drivers (it requires Vulkan 1.1 support). For example, my intel laptop with a IGPU do not support this function because it is stuck at `Vulkan 1.0.43`.

### Feature: shaderStorageBufferArrayDynamicIndexing

By default, dynamic indexing is not supported in shaders. This can be fixed by enabling `shaderStorageBufferArrayDynamicIndexing` or `shaderUniformBufferArrayDynamicIndexing` in the device feature. These features are widely supported.

Note that because write access is required, the shaders only use `buffer block` and not `uniform block`. Dynamic indexing for uniforms need not to be enabled.

### Feature: shaderFloat64

I like to send the time to my shaders using a `double` instead of a `float`. This is a personal preference and this not required for this demo.

## Buffers creation

The next important step is the [buffers allocation](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L869) and [state buffer allocation](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L967).

In this step, three buffers are allocated. One for the vertex and indices, another one for the game data, and one last for the shared game state. Because they will be exposed in the shaders, the usage flags `VK_BUFFER_USAGE_STORAGE_BUFFER_BIT` must be added.

Because the vertex attributes and the game data will only the GPU will access the data, the buffer can be safely backed by `MEMORY_PROPERTY_DEVICE_LOCAL_BIT` memory.

The state buffer must be backed by `MEMORY_PROPERTY_HOST_VISIBLE_BIT` memory because it will be updated from the host.

Two very important things to remember

* By default, the initial buffer content is `undefined`. This means that it must be manually zeroed.
* The buffer bindings must be aligned to `minStorageBufferOffsetAlignment`. Otherwise the program will likely only work on your hardware vendor.

## Data layout

The compute shaders expose 4 buffer bindings:

* [indices](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L74) - For the indices data
* [attributes](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L78) - For the attribute data
* [game](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L82) - For the game data
* [state](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L99) - For the shared state

The vertex shader only expose the game buffer

* [game](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.vert#L65)

In the vertex shader, the game buffer binding *must* be marked as `readonly`, otherwise the shader will use atomics to read from it, which will crash the program because this is a feature that must be enabled in the device.

Every bindings use the `std430` layout (which is only avaible on `buffer` bindings). This allow the data to be tighly packed in arrays and structs. The default `std140` can introduce a lot of unnecessary padding. for more information see [this](https://www.khronos.org/opengl/wiki/Interface_Block_(GLSL)#Memory_layout).

Using storage buffers also remove the buffer range limit. Even on Nvdia hardware (which is usually the most restrictive). For example, according to GPUINFO ([link](https://vulkan.gpuinfo.org/displayreport.php?id=5661#limits)) a NVDIA GTX 1070 maximum range for a uniform buffer is `65536` bytes, but for storage buffers it is `4294970000` bytes.

This makes life a bit easier when [defining the write sets](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1133) because `VK_WHOLE_SIZE` can be used without worries.

## Writing meshes from compute shaders

Attributes and indices data must be defined as array in the shader. [Here is an example](https://github.com/gabdube/asteroids-shader/blob/master/asteroids_init.comp#L288). A loop must then iterate over the values and save them in the buffers.

Not only is this very painful to write, it also produce very bloated SPRIV code. It was fine for this example  because the meshes are very simple, but uploading a complex mesh would probably make the SPIRV binary explode in size.

Also, this is probably very slow. Luckily, mesh uploading is only in the initialization compute shader.

## Managing draw call from compute shaders

Both the count buffer and the draw parameter buffer are exposed in the shader inside the `game` binding. The draw count is exposed in `game.drawCount` and the draw parameters are exposed in `game.objects`. Each draw parameter is associated to a game object. By [sending a stride](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.py#L1604) that is bigger than the `VkDrawIndexedIndirectCommand` structure, it is possible to append extra information to the draw commands.

Adding a draw call to the program is done by increasing the count buffer and appending a new value to the game obejct array.

Removing an object is done by setting the `VkDrawIndexedIndirectCommand.indexCount` to 0 and flag this particular [draw command as "unused"](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L140) using the extra data.

Don't even think about moving the `VkDrawIndexedIndirectCommand` in the array. Seriously. I spent 5 hours trying to debug only to rewrite the whole thing because I coudn't understand why the draw command were being corrupted.

### Pointing to the right matrix in the vertex shader

The last important thing to remember is that the vertex shader do not have any clue about how to process that draw commands managed by the compute shader. Hopefully for us, the `VkDrawIndexedIndirectCommand` can be self referencing using the `firstInstance` field.

Basically, setting the `firstInstance` field the current game object index ([example](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.comp#L179)) means that the vertex shader [will be able to fetch](https://github.com/gabdube/asteroids-shader/blob/master/asteroids.vert#L83) the right game object using the `gl_InstanceIndex` value.

Note that `gl_InstanceIndex` is only available in Vulkan shader. For more information, see the accepted answer [here](https://stackoverflow.com/questions/35638512/instanced-glsl-shaders-in-vulkan).

## Debugging game logic from shaders

*this page was intentionally left blank*

## Conclusion

Because of the serious limitations: everything must fit in GPU memory, debugging is next to impossible, textures must use an atlas, etc. Game logic should be stay a CPU task. Nevertheless, this project was a lot of fun to code. 10/10 would not program DOOM.
