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
    return new Promise<void>(() => {
      const animationName = `${prefix}${animation}`;
      const element = node as HTMLElement;
      element.classList.remove(animationName);
      element.classList.add(`${prefix}animated`, animationName);
      element.style.animationDelay = `${index * delayStep}ms`;
    });
  });

  return Promise.all(animationPromises);
};