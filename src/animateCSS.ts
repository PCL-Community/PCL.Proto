export const animateCss = (element: string, animation: string, prefix = 'animate__') =>
  new Promise((resolve, reject) => {
    const animationName = `${prefix}${animation}`;
    const node = document.querySelector(element) as HTMLElement;

    node.classList.add(`${prefix}animated`, animationName);

    function handleAnimationEnd(event: AnimationEvent) {
      event.stopPropagation();
      node.classList.remove(`${prefix}animated`, animationName);
      resolve('Animation ended');
    }

    node.addEventListener('animationend', handleAnimationEnd, { once: true });
  });


export const animateCssFor = (elements: NodeListOf<Element>, animation: string, delayStep = 20, prefix = 'animate__') => {
  const animationPromises = Array.from(elements).map((node, index) => {
    return new Promise<void>((resolve) => {
      const animationName = `${prefix}${animation}`;
      const element = node as HTMLElement;

      // 确保之前的动画类已被移除
      element.classList.remove(`${prefix}animated`, animationName);

      // 使用 requestAnimationFrame 确保在下一帧添加类
      requestAnimationFrame(() => {
        element.classList.add(`${prefix}animated`, animationName);
        element.style.animationDelay = `${index * delayStep}ms`;

        const handleAnimationEnd = (event: AnimationEvent): void => {
          event.stopPropagation();
          element.classList.remove(`${prefix}animated`, animationName);
          element.style.animationDelay = '';
          resolve();
        };

        element.addEventListener('animationend', handleAnimationEnd, { once: true });
      });
    });
  });

  return Promise.all(animationPromises);
};