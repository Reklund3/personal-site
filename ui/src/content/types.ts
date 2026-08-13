export interface Bullet {
  /** Optional bold inline label rendered before the text, e.g. "Type Systems: …" */
  label?: string;
  text: string;
}

export interface AboutSubSection {
  id: string;
  heading: string;
  intro: string[];
  bullets: Bullet[];
  outro: string[];
}

export interface AboutContent {
  intro: string;
  sections: AboutSubSection[];
}

export interface SkillItem {
  /** Also the color key — see `skillChipColor`, which derives the swatch from it. */
  name: string;
}

export interface SkillCategory {
  label: string;
  items: SkillItem[];
}

export interface SkillsContent {
  soft: string[];
  /** Heading for the soft-skill card, e.g. "Soft Skills". */
  softLabel: string;
  categories: SkillCategory[];
}

export interface ExperienceItem {
  title: string;
  company: string;
  /** Pre-formatted, e.g. "03/2022 – Present" (en dash). */
  dates: string;
  bullets: string[];
}

export interface PortfolioProject {
  title: string;
  /** Rendered as a subdued line under the title — open-source entries only. */
  subheader?: string;
  paragraphs: string[];
  link: string;
  /** Defaults to "View on GitHub" when absent, e.g. "View PR #28871". */
  linkLabel?: string;
}

export interface PortfolioContent {
  personal: PortfolioProject[];
  openSource: PortfolioProject[];
}

export interface NavItem {
  /** Matches the section element id and drives the scroll-spy. */
  id: 'about' | 'skills' | 'experience' | 'education' | 'portfolio';
  label: string;
}

export interface SiteContent {
  profile: {
    name: string;
    title: string;
    tagline: string;
    github: string;
    linkedin: string;
  };
  navItems: NavItem[];
  about: AboutContent;
  skills: SkillsContent;
  experience: ExperienceItem[];
  /** Two pre-composed lines; the design renders them as plain centered text. */
  education: string[];
  portfolio: PortfolioContent;
}
