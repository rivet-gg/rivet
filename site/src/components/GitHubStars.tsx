import { useState, useEffect } from 'react';
import { Icon, faGithub } from '@rivet-gg/icons';

interface GitHubStarsProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  repo?: string;
}

function formatNumber(num: number): string {
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}k`;
  }
  return num.toString();
}

export async function GitHubStars({ repo = 'rivet-gg/rivet', ...props }: GitHubStarsProps) {
  try {
    const response = await fetch(`https://api.github.com/repos/${repo}`);
    const data = await response.json();
    const { stargazers_count: stars } = data;
      return (
    <a
      href={`https://github.com/${repo}`}
      target='_blank'
      rel='noreferrer'
      {...props}>
      <Icon icon={faGithub} /> <span className="hidden md:inline">{stars ? `${formatNumber(stars)} Stars` : 'GitHub'}</span>
    </a>
  );
  } catch (err) {
    console.error('Failed to fetch stars', err);
    return null;
  }
} 
