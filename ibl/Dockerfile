FROM --platform=linux/amd64 ubuntu:resolute

RUN apt update && apt install -y \
    git \
    curl \
    xz-utils \
    libarchive-tools \
    build-essential \
    cmake \
    libglvnd-dev \
    libglx-mesa0 \
    libvulkan-dev \
    vulkan-tools \
    libxcomposite1 \
    libxdamage1 \
    libxkbcommon-dev \
    libxcb-xinput0 \
    libxcb-xinerama0 \
    libxcb-cursor-dev \
    libxi6 \
    libxrandr2 \
    libxrender1 \
    libxtst6 \
    mesa-vulkan-drivers \
    xvfb

WORKDIR /ibl

ENV VULKAN_SDK=/ibl/vulkan/1.4.363.1/x86_64
ENV PATH=$VULKAN_SDK/bin:$PATH
ENV LD_LIBRARY_PATH=$VULKAN_SDK/lib/VulkanLoader/lib
ENV VK_ADD_LAYER_PATH=$VULKAN_SDK/share/vulkan/explicit_layer.d
ENV PKG_CONFIG_PATH=$VULKAN_SDK/share/pkgconfig:$VULKAN_SDK/lib/pkgconfig
ENV CMAKE_PREFIX_PATH=$VULKAN_SDK:$VULKAN_SDK/lib/VulkanLoader
# tar broken on Orb, use bsdtar https://github.com/orbstack/orbstack/issues/2588
RUN mkdir vulkan && cd vulkan && \
    curl -O https://sdk.lunarg.com/sdk/download/1.4.363.1/linux/vulkansdk-linux-x86_64-1.4.363.1.tar.xz && \
    bsdtar xf ./vulkansdk-linux-x86_64-1.4.363.1.tar.xz

RUN git clone --depth 1 --recurse-submodules --shallow-submodules https://github.com/KhronosGroup/glTF-IBL-Sampler.git
RUN curl -o ktx.deb -L https://github.com/KhronosGroup/KTX-Software/releases/download/v5.0.0-rc2/KTX-Software-5.0.0-rc2-Linux-x86_64.deb && \
    apt install -y ./ktx.deb
ENV CMAKE_POLICY_VERSION_MINIMUM=3.5
RUN mkdir build && cd build && cmake ../glTF-IBL-Sampler && make

ENTRYPOINT ["/usr/bin/xvfb-run", "/ibl/build/cli"]
