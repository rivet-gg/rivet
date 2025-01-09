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

export function GitHubStars({ repo = 'rivet-gg/rivet', ...props }: GitHubStarsProps) {
  const [stars, setStars] = useState(0);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`https://api.github.com/repos/${repo}`);
        const data = await response.json();
        setStars(data.stargazers_count);
      } catch (err) {
        console.error('Failed to fetch stars', err);
      }
    };

    fetchData();
  }, [repo]);

  return (
    <a
      href={`https://github.com/${repo}`}
      target='_blank'
      rel='noreferrer'
      {...props}>
      <Icon icon={faGithub} /> <span className="hidden md:inline">{stars ? `${formatNumber(stars)} Stars` : 'GitHub'}</span>
    </a>
  );
} 