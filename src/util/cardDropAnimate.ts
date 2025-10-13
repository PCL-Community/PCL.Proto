import { animate, stagger, type ElementOrSelector } from "motion-v";

export default (element: ElementOrSelector) => {
  animate(
    element,
    { y: [-20, 0], opacity: [0, 1] },
    {
      delay: stagger(0.06, { startDelay: 0 }),
      type: 'spring',
      duration: 0.6,
      bounce: 0.49,
    },
  )
}