<template>
  <div class="wrapper">
    <img alt="PCL.Proto logo" class="logo" src="/PCL.Proto.svg" width="125" height="125" />
    <div class="greetings">
      <h1 class="blue"><a href="https://github.com/PCL-Community/PCL.Proto.git">PCL.Proto</a></h1>
      <h3>
        Here's a prototype of
        <a href="https://github.com/Hex-Dragon/PCL2/" target="_blank" rel="noopener">PCL</a>.
        <br />For deatiled information,
        head to <a href="https://js.design/f/QVPQRY?p=zX2rcVk6Cy&mode=design">即时设计</a>.
      </h3>
    </div>
  </div>

</template>

<style scoped>
.logo {
  display: block;
  margin: 0 2rem 0 0;
}

.wrapper {
  line-height: 1.5;
  max-height: 100vh;
  display: flex;
  /* display: none; */
  place-items: center;
  margin: 0 2rem 2rem 2rem;
}

h1 {
  font-weight: 500;
  font-size: 2.6rem;
  position: relative;
  top: -10px;
}

h3 {
  font-size: 1.2rem;
  padding-left: 0.5rem;
}

.greetings h1,
.greetings h3 {
  text-align: left;
}
</style>
