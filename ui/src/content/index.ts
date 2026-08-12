import type { SiteContent } from './types';

export const CONTENT: SiteContent = {
  profile: {
    name: 'Robert Eklund',
    title: 'Software Engineer',
    tagline: 'Building scalable systems with functional programming principles. Passionate about type safety, DevOps automation, and mentoring engineers.',
    github: 'https://github.com/Reklund3',
    linkedin: 'https://www.linkedin.com/in/robert-eklund-64302976/',
  },
  navItems: [
    { id: 'about', label: 'About' },
    { id: 'skills', label: 'Skills' },
    { id: 'experience', label: 'Experience' },
    { id: 'education', label: 'Education' },
    { id: 'portfolio', label: 'Portfolio' },
  ],
  about: {
    intro: 'Software engineer with a unique background spanning finance, DevOps, and full-stack development. Passionate about type systems, functional programming, and building scalable infrastructure. Driven by performance optimization, mentorship, and fostering inclusive engineering cultures.',
    sections: [
      {
        id: 'background',
        heading: 'Background',
        intro: [
          'Leaving a secure finance career to pursue software engineering felt like jumping off a cliff, but it was a jump I had to make. My background in finance instilled analytical and critical thinking skills that proved invaluable when refactoring codebases and reducing technical debt. After completing Austin Coding Academy, I landed a role at Cloud Imperium Games as an Associate DevOps Engineer, diving headfirst into Scala, event-driven architecture, and distributed systems.',
        ],
        bullets: [],
        outro: [],
      },
      {
        id: 'journey',
        heading: 'Technical Journey',
        intro: [
          'Coming from C# and JavaScript, Scala opened up a new world of functional programming concepts:',
        ],
        bullets: [
          {
            label: 'Functional Programming',
            text: 'Monads, pure functions, higher-order functions, and tail recursion optimization',
          },
          {
            label: 'Type Systems',
            text: 'Leveraging types to express guarantees and make invalid states unrepresentable',
          },
          {
            label: 'Advanced Libraries',
            text: 'Refined for compile-time validation, Cats for functional abstractions, NonEmpty collections',
          },
          {
            label: 'Language Evolution',
            text: 'Transitioned from Scala 2 implicits to Scala 3 givens',
          },
        ],
        outro: [
          'These concepts fundamentally changed how I approach software design, leading me to mentor team members on type-driven development and proper test coverage beyond golden path scenarios.',
        ],
      },
      {
        id: 'achievements',
        heading: 'Key Achievements',
        intro: [],
        bullets: [
          {
            text: 'Modernized legacy systems and migrated a monolithic internal webservice to microservices architecture',
          },
          {
            text: 'Contributed to both ScalaJS/Play frontend and Lagom/Akka backend implementations',
          },
          {
            text: 'Developed critical internal Scala libraries used across multiple team services',
          },
          {
            text: 'Optimized application performance through benchmarking and Docker container size reduction',
          },
        ],
        outro: [],
      },
      {
        id: 'devops',
        heading: 'DevOps & Infrastructure',
        intro: [
          'Led development of microservices CI/CD pipelines and infrastructure modernization:',
        ],
        bullets: [
          {
            text: 'Deployed and configured GitLab runners for automated builds and deployments',
          },
          {
            text: 'Managed GitLab container registry for Docker images and package registry for Helm charts',
          },
          {
            text: 'Orchestrated EKS deployments, upgrading from Helm 2 to Helm 3',
          },
          {
            text: 'Implemented GitOps workflows with ArgoCD for automated deployments and rollback capabilities',
          },
        ],
        outro: [],
      },
      {
        id: 'mentorship',
        heading: 'Mentorship & Collaboration',
        intro: [
          "Beyond technical work, I'm passionate about helping others grow and fostering collaborative environments. My favorite aspect of software engineering is working with talented, passionate people who share knowledge freely.",
          'I believe explaining concepts to others refines our own understanding and strengthens the team. More importantly, we should actively work to foster cultures of inclusion, continuous learning, and mutual growth.',
        ],
        bullets: [],
        outro: [],
      },
      {
        id: 'future',
        heading: 'Future Focus',
        intro: [
          "The rise of LLMs and AI tools has captured my attention. Tools like Cursor, Windsurf, and JetBrains' AI assistants are revolutionizing how we build software, enabling faster prototyping, better context search, and exploration of alternative solutions.",
          "I look forward to the challenges ahead and opportunities to collaborate with like-minded technologists. If you're interested in connecting, reach out via the Contact button or find me on LinkedIn.",
        ],
        bullets: [],
        outro: [],
      },
    ],
  },
  skills: {
    soft: [
      'Team Player',
      'Problem Solver',
      'Strategic Thinking',
      'Communication',
      'Leadership',
    ],
    categories: [
      {
        label: 'Programming Languages',
        items: [
          { name: 'Scala', color: '#DC322F' },
          { name: 'Rust', color: '#CE422B' },
          { name: 'Java', color: '#007396' },
          { name: 'C#', color: '#239120' },
          { name: 'C++', color: '#00599C' },
          { name: 'Go', color: '#00ADD8' },
        ],
      },
      {
        label: 'Cloud & Infrastructure',
        items: [
          { name: 'Docker', color: '#2496ED' },
          { name: 'Kubernetes', color: '#326CE5' },
          { name: 'Helm', color: '#0F1689' },
          { name: 'Argo', color: '#EF7B4D' },
          { name: 'AWS EKS', color: '#FF9900' },
          { name: 'GKE', color: '#4285F4' },
        ],
      },
      {
        label: 'Frameworks & Tools',
        items: [
          { name: 'Pekko', color: '#1976d2' },
          { name: 'Akka', color: '#1976d2' },
          { name: 'Play', color: '#92D13D' },
          { name: 'Okta', color: '#007DC1' },
          { name: 'ScalaJS', color: '#DC322F' },
          { name: 'Diode', color: '#1976d2' },
          { name: 'Git', color: '#F05032' },
          { name: 'GitLab', color: '#FC6D26' },
          { name: 'GitHub', color: '#181717' },
        ],
      },
      {
        label: 'Databases',
        items: [
          { name: 'PostgreSQL', color: '#336791' },
          { name: 'MySQL', color: '#4479A1' },
          { name: 'Cassandra', color: '#1287B1' },
          { name: 'Kafka', color: '#231F20' },
        ],
      },
      {
        label: 'DevOps & CI/CD',
        items: [
          { name: 'GitHub Actions', color: '#2088FF' },
          { name: 'GitLab CI', color: '#FC6D26' },
          { name: 'Helm Charts', color: '#0F1689' },
          { name: 'Docker Pipelines', color: '#2496ED' },
        ],
      },
      {
        label: 'Other Tools',
        items: [
          { name: 'JetBrains Suite', color: '#5b5b5b' },
          { name: 'AWS', color: '#FF9900' },
          { name: 'Miro', color: '#c9a800' },
          { name: 'Jira', color: '#0052CC' },
          { name: 'Atlassian', color: '#0052CC' },
        ],
      },
    ],
  },
  experience: [
    {
      title: 'Senior Micro-Service Engineer',
      company: 'Cloud Imperium Games',
      dates: '03/2022 – Present',
      bullets: [
        'Led code reviews for full team across all domains, infrastructure, platform code, and frontend.',
        'Held cross-functional meetings with stakeholders to gather technical requirements, reducing ambiguity and ensuring delivery on time. Additionally, managed feature request and bug reports.',
        'Implemented Authorization (AuthZ) and Authentication (AuthN) for service offerings using Attribute-Based Access Control (ABAC), securing systems against unauthorized access.',
        'Developed and maintained internal Scala libraries depended on by a large portion of the teams internal services.',
        'Investigated FFI implementations with Bindgen to work on a port of Pekko to Rust. This would allow for keeping the current Scala tests and strangle thorn Scala/java into Rust.',
      ],
    },
    {
      title: 'Micro-Service Engineer',
      company: 'Cloud Imperium Games',
      dates: '03/2021 – 03/2022',
      bullets: [
        'Proactively mentored junior engineers in system design, containerization, and deployment best practices.',
        'Designed and deployed Kubernetes-based micro-services, increasing scalability and fault tolerance.',
        'Enhanced deployment automation using Helm, reducing deployment times while maintaining system reliability.',
        'Contributed and maintained full-stack efforts on the teams internal offerings.',
      ],
    },
    {
      title: 'DevOps Engineer',
      company: 'Cloud Imperium Games',
      dates: '03/2020 – 03/2021',
      bullets: [
        'Contributed to the development and maintenance internal full stack software offering.',
        'Developed CI/CD pipelines in GitLab to automate deployments, reducing failures and build times.',
        'Built and maintained scalable internal Docker images to improve productivity for integrated testing.',
      ],
    },
    {
      title: 'Associate DevOps Engineer',
      company: 'Cloud Imperium Games',
      dates: '03/2019 – 03/2020',
      // Intentionally empty: the resume lists this role as a header with no bullets,
      // so the site matches it rather than shipping a "Coming soon." placeholder.
      // The entry still earns its place — it is the start of the progression through
      // four roles at CIG. Fill it in if there is something worth saying.
      bullets: [],
    },
  ],
  education: [
    "Texas State University, San Marcos, Tx — Master's in Accounting Information Systems",
    "Texas State University, San Marcos, Tx — Bachelor's in Accounting",
  ],
  portfolio: {
    personal: [
      {
        title: 'Zero2Prod',
        paragraphs: [
          'This is the website you are currently on. It is a React/Typescript/Material UI application.',
          'It is a simple portfolio website that I created to showcase my skills and projects.',
        ],
        link: 'https://github.com/Reklund3/personal-site',
      },
      {
        title: 'Posts',
        paragraphs: [
          'Akka post service to manage posts and comments. This project was started as a learning exercise to learn more about Akka and Akka HTTP. It evolved into learning about gRPC.',
          'I intend to circle back to this project and convert it to Pekko.',
        ],
        link: 'https://gitlab.com/Reklund3/posts',
      },
      {
        title: 'Posts-App',
        paragraphs: [
          'A project I created to explore the Tauri framework. This is a simple app that uses the Posts service and various crates to integrate with gRPC transport to the Posts service.',
          "I don't currently have any additional plans for this project but it has been a great learning experience.",
        ],
        link: 'https://github.com/Reklund3/posts-app/tree/init',
      },
    ],
    openSource: [
      {
        title: 'Akka ActorTestkit',
        subheader: 'Factory Methods Enhancement',
        paragraphs: [
          'Contributed factory methods to the Akka ActorTestkit framework, improving distributed system test workflows.',
          'Enhanced functionality for Typed Actor TestKit, making it easier for developers to write comprehensive tests for actor-based systems.',
        ],
        link: 'https://github.com/akka/akka/pull/28871',
        linkLabel: 'View PR #28871',
      },
    ],
  },
};
